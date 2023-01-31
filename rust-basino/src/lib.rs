//! A crate to work with custom user stacks on AVR
#![warn(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

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
#[repr(packed)]
pub struct Stack {
    // The address of the bottom of the stack
    // pub stack_bottom_sentinel: u8,
    /// The actual stack array which holds the data
    pub stack: [u8; 32],
    /// The address of the top of the stack
    pub stack_top_sentinel: *mut u8,
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

/// Dummy location needed because of DEVICE_PERIPHERALS
/// Linker scripts aren't working on AVR, so we can't have fine-grained control
/// over memory.  This isn't an ideal solution, but it works for now.
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
pub static mut BASINO_STACK: Stack = Stack {
    stack: [0; 32],
    stack_top_sentinel: 0 as *mut u8,
};

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

    /// Initialize the stack
    pub fn basino_stack_init(memory_start: *mut u8, stack_bottom: *mut u8, stack_size: u8) -> u8;

    /// Push a value onto the stack
    pub fn basino_stack_push(value: u8) -> u8;

    /// Pop a value from the stack
    pub fn basino_stack_pop(result: *mut u8) -> u8;

    /// Get the address of the bottom of the stack
    pub fn basino_get_basino_stack_bottom() -> u16;

    /// Get the address of the top of the stack
    pub fn basino_get_basino_stack_top() -> u16;

    /// Get the address of the top of the stack sentinel
    pub fn basino_get_basino_stack_top_sentinel() -> u16;

    /// Get the stack size
    pub fn basino_get_basino_stack_size() -> u8;

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
}
