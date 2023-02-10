//! A crate to work with custom user stacks on AVR
#![warn(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]
#![feature(ptr_from_ref)]

/// Error data types
pub mod error;

/// Queue functions and data structures
pub mod queue;

/// Stack functions and data structures
pub mod stack;

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
pub struct Stack {
    /// The actual stack array which holds the data
    pub data: *mut u8,

    /// A sentinel to make comparisons against the top simpler
    pub top_sentinel: *mut u8,

    /// The stack bottom
    pub bottom: *mut u8,

    /// The stack top
    pub top: *mut u8,
}

/// The Queue data structure
pub struct Queue {
    /// The actual queue array which holds the data
    pub queue: [u8; 32],
    // /// The address of the start of the queue
    // pub start: *mut u8,
    // /// The address of the end of the queue
    // pub end: *mut u8,
    // /// The current head in the queue
    // /// The head points to the location of the current item to be
    // /// returned with a get operation.
    // pub head: *mut u8,
    // /// The last head position in the queue
    // pub last_head: *mut u8,
    // /// The current tail of the queue
    // /// The tail points to the the location where the next item will
    // /// be put.
    // pub tail: *mut u8,
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
/// Add the #[used] attribute to keep this static even if it's not
/// used in the program.
#[link_section = ".ram2bss"]
#[used]
pub static mut DEVICE_PERIPHERALS_SPACE: u8 = 0;

// Putting everything into a single structure is correctly allocating the data
// in the .ram2bss section now, but not in the right location according to our
// memory.x linker script
/// The stack object we pass into the C / assembly code to store data
#[link_section = ".ram2bss"]
pub static mut BASINO_STACK: Option<Stack> = None;

/// The stack buffer that stores the data C / assembly code to store data
/// The length of the stack is the length of this buffer minus one.
/// An additional byte is used as a top sentinel
///
/// There are no references to memory alignment requirements in the
/// Atmel data sheets DS40002061B (ATmega48A/PA/88A/PA/168A/PA/328/P)
/// and DS40002198B (AVR® Instruction Set Manual)
///
/// Several unofficial references online make the point that 16-bit
/// memory accesses are composed of two 8-bit accesses.  Even still, we'll
/// align two bytes here.  The extra byte is assumed unneeded.
#[link_section = ".ram2bss"]
pub static mut BASINO_STACK_BUFFER: [u8; 33] = [0; 33];

/// The queue object we pass into the C / assembly code to store data
#[link_section = ".ram2bss"]
pub static mut BASINO_QUEUE_DATA: [u8; 32] = [0; 32];

/// The queue object we pass into the C / assembly code to store data
/// This should be initialized by the code before being used
#[link_section = ".ram2bss"]
pub static mut BASINO_QUEUE: Queue = Queue {
    queue: unsafe { BASINO_QUEUE_DATA },
};

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
    pub fn basino_stack_init(stack: *mut Stack, top: *mut u8, bottom: *mut u8) -> u8;

    /// Push a value onto the stack
    pub fn basino_stack_push(stack: *const Stack, value: u8) -> u8;

    /// Pop a value from the stack
    pub fn basino_stack_pop(stack: *const Stack, result: *mut u8) -> u8;

    /// Get the address of the bottom of the stack
    pub fn basino_get_basino_stack_bottom(stack: *const Stack) -> *const u8;

    /// Get the address of the top of the stack
    pub fn basino_get_basino_stack_top(stack: *const Stack) -> *const u8;

    /// Get the address of the top of the stack sentinitel
    pub fn basino_get_basino_stack_top_sentinel(stack: *const Stack) -> *const u8;

    // /// Get the stack size
    // pub fn basino_get_basino_stack_size(stack: *const Stack) -> u;

    /// Initialize the queue
    pub fn basino_queue_init(start: *mut u8, end: *mut u8) -> u8;

    /// Put an item into the queue
    pub fn basino_queue_put(value: u8) -> u8;

    /// Get an item from the queue
    pub fn basino_queue_get(result: *mut u8) -> u8;

    // Info functions

    /// Get the start of the queue
    pub fn basino_queue_get_queue_start() -> *const u16;
    /// Get the end of the queue
    pub fn basino_queue_get_queue_end() -> *const u16;
    /// Get the current head of the queue
    pub fn basino_queue_get_head() -> *const u16;
    /// Get the last head of the queue
    pub fn basino_queue_get_last_head() -> *const u16;
    /// Get the current tail of the queue
    pub fn basino_queue_get_tail() -> *const u16;
}

/// Test module for the top-level Forth system
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
