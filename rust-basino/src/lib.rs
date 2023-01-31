//! A crate to work with custom user stacks on AVR
#![warn(missing_docs)]
#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

/// Error data types
pub mod error;

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
    pub stack: [u8; 128],
    /// The address of the top of the stack
    pub stack_top_sentinel: *mut u8,
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
    stack: [0; 128],
    stack_top_sentinel: 0 as *mut u8,
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
}
