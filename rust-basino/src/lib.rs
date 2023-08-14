//! A crate to work with custom user stacks on AVR
#![warn(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(ptr_from_ref)]

use avr_device::interrupt::Mutex;
use core::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    marker::PhantomData,
};
use ufmt::{uDebug, uWrite};

/// Error data types
pub mod error;

/// Queue functions and data structures
pub mod queue;

/// Stack functions and data structures
pub mod stack;

/// Interpretive Language functions and data structures
pub mod il;

use crate::il::Interpreter;

/// Create a type alias to simplify function parameters
pub type Usart = arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>;

/// A handle to an array to manage lifetimes and concurrency
pub struct ArrayHandle<'a, T> {
    /// Pointer to the array
    pub ptr: *mut T,
    /// Length of the array

    /// This is the number of elements the array can hold, not
    /// necessarily the number of bytes.
    ///
    /// For example, for the following array:
    /// let mut arr: [u8; 4] = [0; 4];
    /// len would be 4
    ///
    /// For the following array:
    /// let mut arr: [u16; 4] = [0; 4];
    /// len would also be 4
    pub len: usize,
    /// We want to have a lifetime on an ArrayHandle tied to the data
    pub _marker: PhantomData<&'a T>,
}

impl<'a, T> ArrayHandle<'a, T> {
    /// Create a new ArrayHandle from an array pointer and length.
    ///
    /// ptr points to an array of data of type T
    /// len is the length of the array, the number of elements of type
    /// T it can hold.
    ///
    /// # Safety
    ///
    /// ptr must point to a valid array.  The ptr must be an array of
    /// length len.  It is the responsibility of the caller to
    /// allocate and deallocate this array.  The array must live as
    /// long as the ArrayHandle.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::ArrayHandle;
    ///
    /// let mut arr: [u8; 4] = [0; 4];
    /// let _handle = ArrayHandle::new(arr.as_mut_ptr(), arr.len());
    /// ```
    pub fn new(ptr: *mut T, len: usize) -> Self {
        ArrayHandle {
            ptr,
            len,
            _marker: PhantomData,
        }
    }
}

// We can only have one link section for the same library file
// Otherwise it tries to include the library twice, so C externs are
// consolidated here in lib.rs
// I don't know if there's any guarantee about ordering of data,
// unless that data is stuffed in a structure.  Because of the way we
// do stack manipulations, the stack must be placed in memory before
// the variables.
/// The basic stack structure.  Stores the stack and the variables to
/// hold metadata about the stack.
/// Due to the way we do stack operations, the order here matters.
/// It's a fragile solution, and may want to be refactored.
///
/// The stack_top_sentinel is located at top of the stack, it contains
/// the address of the top and provides padding for the stack.
#[repr(C)]
pub struct Stack<'a> {
    /// The actual stack array which holds the data
    pub data: *mut u8,

    /// A sentinel to make comparisons against the top simpler
    pub top_sentinel: *mut u8,

    /// The stack bottom
    pub bottom: *mut u8,

    /// The stack top
    pub top: *mut u8,

    /// We want this structure to last as long as the lifetime of the array
    /// its based on.
    _marker: PhantomData<&'a u8>,
}

impl<'a> Debug for Stack<'a> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "  stack data 0x{}", &(self.data as usize))?;
        write!(f, ", top sentinel: 0x{}", &(self.top_sentinel as usize))?;
        write!(f, ", bottom: 0x{}", &(self.bottom as usize))?;
        write!(f, ", top: 0x{}", &(self.top as usize))?;
        write!(f, ", head item: 0x{}", &(unsafe { *(self.top) }))
    }
}

impl<'a> uDebug for Stack<'a> {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        f.write_str("  stack data 0x")?;
        ufmt::uDisplay::fmt(&(self.data as usize), f)?;
        f.write_str(", top sentinel: 0x")?;
        ufmt::uDisplay::fmt(&(self.top_sentinel as usize), f)?;
        f.write_str(", bottom: 0x")?;
        ufmt::uDisplay::fmt(&(self.bottom as usize), f)?;
        f.write_str(", top: 0x")?;
        ufmt::uDisplay::fmt(&(self.top as usize), f)?;
        f.write_str(", head item: 0x")?;
        ufmt::uDisplay::fmt(&(unsafe { *(self.top) }), f)
    }
}

/// The Queue data structure
#[repr(C)]
pub struct QueueObj<'a> {
    /// The actual queue array which holds the data
    pub queue: *mut u8,
    /// The address of the start of the queue
    pub start: *const u8,
    /// The address of the end of the queue
    pub end: *const u8,
    /// The current head in the queue
    /// The head points to the location of the current item to be
    /// returned with a get operation.
    pub head: *mut u8,
    /// The last head position in the queue
    pub last_head: *mut u8,
    /// The current tail of the queue
    /// The tail points to the the location where the next item will
    /// be put.
    pub tail: *mut u8,

    /// We want this structure to last as long as the lifetime of the array
    /// its based on.
    _marker: PhantomData<&'a u8>,
}

impl<'a> uDebug for QueueObj<'a> {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        f.write_str("  queue head 0x")?;
        ufmt::uDisplay::fmt(&(self.head as usize), f)?;
        f.write_str(", last head: 0x")?;
        ufmt::uDisplay::fmt(&(self.last_head as usize), f)?;
        f.write_str(", head item: 0x")?;
        ufmt::uDisplay::fmt(&(unsafe { *(self.head) }), f)?;
        f.write_str(", tail: 0x")?;
        ufmt::uDisplay::fmt(&(self.tail as usize), f)?;
        f.write_str(", start: 0x")?;
        ufmt::uDisplay::fmt(&(self.start as usize), f)?;
        f.write_str(", end: 0x")?;
        ufmt::uDisplay::fmt(&(self.end as usize), f)
    }
}

/// Queue data structure with length field.
/// This can be simplified after the initial structure refactor is
/// done.
#[repr(C)]
pub struct Queue<'a> {
    /// The actual queue object
    pub queue: QueueObj<'a>,
    /// Length of the queue.
    pub queue_len: usize,
}

impl<'a> uDebug for Queue<'a> {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        f.write_str("  queue head 0x")?;
        ufmt::uDisplay::fmt(&(self.queue.head as usize), f)?;
        f.write_str(", last head: 0x")?;
        ufmt::uDisplay::fmt(&(self.queue.last_head as usize), f)?;
        f.write_str(", head item: 0x")?;
        ufmt::uDisplay::fmt(&(unsafe { *(self.queue.head) }), f)?;
        f.write_str(", tail: 0x")?;
        ufmt::uDisplay::fmt(&(self.queue.tail as usize), f)?;
        f.write_str(", start: 0x")?;
        ufmt::uDisplay::fmt(&(self.queue.start as usize), f)?;
        f.write_str(", end: 0x")?;
        ufmt::uDisplay::fmt(&(self.queue.end as usize), f)
    }
}

/// Technically, on embedded devices with limited memory, even
/// address zero can be used.  Especially on Harvard devices,
/// where interrupts may be in program code and the stack
/// allocated on the heap.
///
/// We can setup a filler byte at the beginning of memory to
/// deal with this.
/// A dummy placeholder so that null pointers aren't accidently used.
#[link_section = ".ram2bss"]
#[used]
pub static mut BASINO_STACK_FILLER: u8 = 1;

/// Dummy location needed because of DEVICE_PERIPHERALS
/// Linker scripts aren't working on AVR, so we can't have fine-grained control
/// over memory.  This isn't an ideal solution, but it works for now.
///
/// This may not be needed with relative linking enabled.
///
/// And it appears (this might be wrong) that Rust is putting our data
/// in the same location as DEVICE_PERIPHERALS from the avr-device
/// crate.
///
/// Add the #\[used\] attribute to keep this static even if it's not
/// used in the program.
#[link_section = ".ram2bss"]
#[used]
pub static mut DEVICE_PERIPHERALS_SPACE: u8 = 0;

// Putting everything into a single structure is correctly allocating the data
// in the .ram2bss section now, but not in the right location according to our
// memory.x linker script

/// The stack buffer that stores the data C / assembly code to store data
/// The length of the stack is the length of this buffer minus one.
/// An additional byte is used as a top sentinel
///
/// There are no references to memory alignment requirements in the
/// Atmel data sheets DS40002061B (ATmega48A/PA/88A/PA/168A/PA/328/P)
/// and DS40002198B (AVR® Instruction Set Manual)
///
/// Several unofficial references online make the point that 16-bit
/// memory accesses are composed of two 8-bit accesses.
#[link_section = ".ram2bss"]
static mut BASINO_STACK_BUFFER: [u8; 33] = [0; 33];

/// Handle to wrap the stack array and allow safe management of it
pub static mut BASINO_STACK_BUFFER_HANDLE: Mutex<RefCell<Option<ArrayHandle<u8>>>> = unsafe {
    Mutex::new(RefCell::new(Some(ArrayHandle {
        ptr: BASINO_STACK_BUFFER.as_mut_ptr(),
        len: BASINO_STACK_BUFFER.len(),
        _marker: PhantomData,
    })))
};

/// The queue object we pass into the C / assembly code to store data
#[link_section = ".ram2bss"]
static mut BASINO_QUEUE_DATA: [u8; 4] = [0; 4];

/// Handle to wrap the queue array and allow safe management of it
pub static mut BASINO_QUEUE_DATA_HANDLE: Mutex<RefCell<Option<ArrayHandle<u8>>>> = unsafe {
    Mutex::new(RefCell::new(Some(ArrayHandle {
        ptr: BASINO_QUEUE_DATA.as_mut_ptr(),
        len: BASINO_QUEUE_DATA.len(),
        _marker: PhantomData,
    })))
};

/// The input queue data
#[link_section = ".ram2bss"]
pub static mut BASINO_INPUT_QUEUE_DATA: [u8; 32] = [0; 32];

/// Handle to wrap the input queue array and allow safe management of it
pub static mut BASINO_INPUT_QUEUE_DATA_HANDLE: Mutex<RefCell<Option<ArrayHandle<u8>>>> = unsafe {
    Mutex::new(RefCell::new(Some(ArrayHandle {
        ptr: BASINO_INPUT_QUEUE_DATA.as_mut_ptr(),
        len: BASINO_INPUT_QUEUE_DATA.len(),
        _marker: PhantomData,
    })))
};

/// The queue object we pass into the C / assembly code to store data
/// This should be initialized by the code before being used
#[link_section = ".ram2bss"]
pub static mut BASINO_QUEUE: Option<Queue> = None;

/// The array for the byte code program
/// This array can be shared between test functions
#[link_section = ".ram2bss"]
pub static mut BASINO_IL_BYTE_CODE_DATA: [u8; 33] = [0; 33];

#[link(name = "basino")]
extern "C" {
    /// Add two 8-bit unsigned integers together
    pub fn basino_add(a: u8, b: u8) -> u16;

    /// Test whether a is greater than b
    /// Returns one if a is greater than b
    /// Return zero if it isn't
    pub fn basino_gt(a: u16, b: u16) -> u8;

    /// Test whether a is greater than or equal to b
    /// Returns one if a is greater than or equal to b
    /// Return zero if it isn't
    pub fn basino_gt_eq(a: u16, b: u16) -> u8;

    // Stack functions

    /// Initialize the stack.
    /// This initializes with the permanent bottom and maximum top.
    /// It sets the current top and bottom to those values.
    /// The top is a top sentinel, it should be one above the stack
    /// size.
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    ///
    /// top and bottom must point to valid memory locations.  The
    /// allocation of the memory is the responsibility of the caller.
    ///
    /// top must be greater than the bottom.
    pub fn basino_stack_init(stack: *mut Stack, top: *mut u8, bottom: *mut u8) -> u8;

    /// Push a value onto the stack
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    pub fn basino_stack_push(stack: *const Stack, value: u8) -> u8;

    /// Pop a value from the stack
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    ///
    /// Result must point to valid memory that is used to store the
    /// result.
    ///
    /// It is the resposiblity of the caller to allocate and
    /// deallocate that memory.
    pub fn basino_stack_pop(stack: *const Stack, result: *mut u8) -> u8;

    /// Get the address of the bottom of the stack
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    pub fn basino_get_basino_stack_bottom(stack: *const Stack) -> *const u8;

    /// Get the address of the top of the stack
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    pub fn basino_get_basino_stack_top(stack: *const Stack) -> *const u8;

    /// Get the address of the top of the stack sentinitel
    ///
    /// # Safety
    ///
    /// The provided stack must not be a null pointer and must point
    /// to valid stack structure.  It is the responsiblity of the
    /// caller to allocate and deallocate the stack structure.
    pub fn basino_get_basino_stack_top_sentinel(stack: *const Stack) -> *const u8;

    // /// Get the stack size
    // pub fn basino_get_basino_stack_size(stack: *const Stack) -> u;

    /// Initialize the queue
    pub fn basino_queue_init(queue: *mut QueueObj, start: *mut u8, end: *mut u8) -> u8;

    /// Put an item into the queue
    pub fn basino_queue_put(queue: *const QueueObj, value: u8) -> u8;

    /// Get an item from the queue
    pub fn basino_queue_get(queue: *const QueueObj, result: *mut u8) -> u8;

    // Info functions

    /// Get the start of the queue
    pub fn basino_queue_get_queue_start(queue: *mut QueueObj, result: *mut u8) -> *const u8;
    /// Get the end of the queue
    pub fn basino_queue_get_queue_end(queue: *mut QueueObj, result: *mut u8) -> *const u8;
    /// Get the current head of the queue
    pub fn basino_queue_get_head(queue: *mut QueueObj, result: *mut u8) -> *const u8;
    /// Get the last head of the queue
    pub fn basino_queue_get_last_head(queue: *mut QueueObj, result: *mut u8) -> *const u8;
    /// Get the current tail of the queue
    pub fn basino_queue_get_tail(queue: *mut QueueObj, result: *mut u8) -> *const u8;

    // Interpretive Language functions

    /// Initialize the interpreter
    /// byte_code_len is the length of the array.  This is used to initialize the
    /// end pointer.
    pub fn basino_il_init(
        interpreter: *mut Interpreter,
        byte_code: *const u8,
        byte_code_len: u16,
        queue: *mut QueueObj,
        stack: *mut Stack,
    ) -> u8;

    /// Get the next bytecode
    pub fn basino_il_get_next_bytecode(interpreter: *mut Interpreter, result: *mut u8) -> u8;
    /// Execute a single command
    pub fn basino_il_run(interpreter: *mut Interpreter) -> u8;

    /// Execute a single command
    pub fn basino_il_exec(interpreter: *mut Interpreter, opcode: u8) -> u8;
}

/// Test module for the top-level Tiny BASIC system
#[allow(unused_imports)]
pub mod tests {
    use crate::{basino_gt, basino_gt_eq};
    use arduino_hal::{
        hal::port::{PD0, PD1},
        pac::USART0,
        port::{
            mode::{Input, Output},
            Pin,
        },
        Usart,
    };
    use core::{arch::asm, fmt::Write};

    /// Write a test result status and message about the test
    ///
    /// writer is the Usart object to write to
    /// test_result is the result of the test:
    ///   if it was true the test was successful
    ///   if it was false the test was a failure
    /// status_msg is a string describing the test
    pub fn write_test_result(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
        test_result: bool,
        status_msg: &str,
    ) {
        if test_result {
            ufmt::uwrite!(writer, "SUCCESS").unwrap();
        } else {
            ufmt::uwrite!(writer, "FAILURE").unwrap();
        }
        ufmt::uwriteln!(writer, " {}\r", status_msg).unwrap();
    }

    /// Run all the tests in this module
    pub fn run_tests(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        test_basino_gt_gt_works(writer);
        test_basino_gt_eq_works(writer);
        test_basino_gt_lt_works(writer);
        test_basino_gt_eq_gt_works(writer);
        test_basino_gt_eq_eq_works(writer);
        test_basino_gt_eq_lt_works(writer);
    }

    /// Test that basino_gt works for greater than
    pub fn test_basino_gt_gt_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let res = unsafe { basino_gt(0x1000, 0x0010) };
        write_test_result(writer, res == 1, "0x1000 should be > 0x0010");
    }

    /// Test that basino_gt works for equal
    pub fn test_basino_gt_eq_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let res = unsafe { basino_gt(0x1111, 0x1111) };
        write_test_result(writer, res == 0, "0x1111 should not be > 0x1111");
    }

    /// Test that basino_gt works for less than
    pub fn test_basino_gt_lt_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let res = unsafe { basino_gt(0x0010, 0x1000) };
        write_test_result(writer, res == 0, "0x0010 should not be > 0x1000");
    }

    /// Test that basino_gt_eq works for greater than
    pub fn test_basino_gt_eq_gt_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let res = unsafe { basino_gt_eq(0x1000, 0x0010) };
        write_test_result(writer, res == 1, "0x1000 should be >= 0x0010");
    }

    /// Test that basino_gt_eq works for equal
    pub fn test_basino_gt_eq_eq_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let res = unsafe { basino_gt_eq(0x1111, 0x1111) };
        write_test_result(writer, res == 1, "0x1111 should be >= 0x1111");
    }

    /// Test that basino_gt_eq works for less than
    pub fn test_basino_gt_eq_lt_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let res = unsafe { basino_gt_eq(0x0010, 0x1000) };
        write_test_result(writer, res == 0, "0x0010 should not be >= 0x1000");
    }
}
