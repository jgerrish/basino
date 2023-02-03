//! Stack functions and data structures
//!
//! This Rust driver is used to test an assembly implementation of a
//! stack on Atmel AVR devices.
//!
//! This crate owns the Struct object, it passes in pointers to the
//! assembly code.
#![warn(missing_docs)]

use arduino_hal::{
    hal::port::{PD0, PD1},
    pac::USART0,
    port::{
        mode::{Input, Output},
        Pin,
    },
    Usart,
};

use crate::{
    basino_get_basino_stack_bottom, basino_get_basino_stack_top,
    basino_get_basino_stack_top_sentinel, basino_stack_init, basino_stack_pop, basino_stack_push,
    Stack, BASINO_STACK_BUFFER,
};

/// Basic functions for a stack
pub trait StackImpl {
    /// Create a new Stack
    fn new(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> Self;

    /// Pop a value from the stack
    fn pop(&mut self) -> Result<u8, u8>;

    /// Push a value onto the stack
    fn push(&mut self, value: u8) -> Result<(), u8>;

    /// Get the size of the stack
    fn size(&mut self) -> u16;

    /// Print a bunch of debugging information about the stack
    fn debug_print(&mut self, writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>);
}

impl StackImpl for Stack {
    fn new(_writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) -> Self {
        let stack_bottom_ptr = unsafe { core::ptr::addr_of_mut!(BASINO_STACK_BUFFER) as *mut u8 };
        let len = unsafe { BASINO_STACK_BUFFER.len() - 1 };

        let stack_top_ptr: *mut u8 = (stack_bottom_ptr as usize + len) as *mut u8;

        let mut stack = Self {
            data: core::ptr::null_mut::<u8>(),
            top_sentinel: core::ptr::null_mut::<u8>(),
            bottom: core::ptr::null_mut::<u8>(),
            top: core::ptr::null_mut::<u8>(),
        };

        unsafe {
            basino_stack_init(
                core::ptr::addr_of_mut!(stack) as *mut Stack,
                stack_top_ptr as *mut u8,
                stack_bottom_ptr as *mut u8,
                stack_top_ptr as *mut u8,
            )
        };

        stack
    }

    fn debug_print(&mut self, writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let res =
            unsafe { basino_get_basino_stack_bottom(core::ptr::addr_of_mut!(*self) as *mut Stack) };

        ufmt::uwriteln!(writer, "basino_get_basino_stack_bottom result: {:?}\r", res).unwrap();

        let res =
            unsafe { basino_get_basino_stack_top(core::ptr::addr_of_mut!(*self) as *mut Stack) };
        ufmt::uwriteln!(writer, "basino_get_basino_stack_top result: {:?}\r", res).unwrap();

        let res = unsafe {
            basino_get_basino_stack_top_sentinel(core::ptr::addr_of_mut!(*self) as *mut Stack)
        };
        ufmt::uwriteln!(
            writer,
            "basino_get_basino_stack_top_sentinel result: {:?}\r",
            res
        )
        .unwrap();

        let res = self.size();
        ufmt::uwriteln!(writer, "size result: {}\r", res).unwrap();
    }

    fn pop(&mut self) -> Result<u8, u8> {
        let mut result: u8 = 0;
        let res =
            unsafe { basino_stack_pop(core::ptr::addr_of_mut!(*self) as *mut Stack, &mut result) };
        match result {
            0 => Ok(res),
            _ => Err(result),
        }
    }

    fn push(&mut self, value: u8) -> Result<(), u8> {
        let res = unsafe { basino_stack_push(core::ptr::addr_of_mut!(*self) as *mut Stack, value) };
        match res {
            0 => Ok(()),
            e => Err(e),
        }
    }

    fn size(&mut self) -> u16 {
        unsafe {
            basino_get_basino_stack_top_sentinel(core::ptr::addr_of_mut!(*self) as *mut Stack)
                as u16
                - basino_get_basino_stack_bottom(core::ptr::addr_of_mut!(*self) as *mut Stack)
                    as u16
        }
    }
}

/// A tests module
/// This doesn't use the standard Rust testing framework.  Instead it's a normal
/// public module that can be called by other systems.
pub mod tests {
    use crate::{
        basino_get_basino_stack_bottom, basino_get_basino_stack_top,
        basino_get_basino_stack_top_sentinel, stack::StackImpl, tests::write_test_result, Stack,
        BASINO_STACK_BUFFER,
    };
    use arduino_hal::{
        hal::port::{PD0, PD1},
        pac::USART0,
        port::{
            mode::{Input, Output},
            Pin,
        },
        Usart,
    };

    /// Run all the tests in this module
    pub fn run_tests(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        test_stack_new_works(writer);
        test_stack_push_works(writer);
        test_stack_empty_pop_fails(writer);
        test_stack_push_full_stack_fails(writer);
        test_stack_push_full_stack_pop_full_works(writer);
    }

    /// Test that initializing the stack works
    pub fn test_stack_new_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let mut stack = Stack::new(writer);

        let expected_size = unsafe { BASINO_STACK_BUFFER.len() - 1 };
        let size = stack.size();

        let expected_bottom: *mut u8 =
            unsafe { core::ptr::addr_of_mut!(BASINO_STACK_BUFFER) as *mut u8 };
        let bottom = unsafe { basino_get_basino_stack_bottom(&stack) };

        let expected_top: *mut u8 = unsafe {
            (core::ptr::addr_of_mut!(BASINO_STACK_BUFFER) as usize + expected_size) as *mut u8
        };
        let top = unsafe { basino_get_basino_stack_top(&stack) };

        let expected_top_sentinel: *mut u8 = unsafe {
            (core::ptr::addr_of_mut!(BASINO_STACK_BUFFER) as usize + expected_size) as *mut u8
        };
        let top_sentinel = unsafe { basino_get_basino_stack_top_sentinel(&stack) };

        write_test_result(
            writer,
            bottom == expected_bottom,
            "initialized stack should have the correct bottom",
        );

        write_test_result(
            writer,
            top == expected_top,
            "initialized stack should have the correct top",
        );

        write_test_result(
            writer,
            top_sentinel == expected_top_sentinel,
            "initialized stack should have the correct top sentinel",
        );

        write_test_result(
            writer,
            size == expected_size as u16,
            "initialized stack should have the correct size",
        );
    }

    /// Test that pushing a value on the stack works
    pub fn test_stack_push_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let mut stack = Stack::new(writer);

        let res = stack.push(5);
        write_test_result(writer, res.is_ok(), "should be able to push value");

        let res = stack.pop();
        write_test_result(writer, res.is_ok(), "should be able to pop value");
        write_test_result(
            writer,
            res.unwrap() == 5,
            "popped value should equal pushed value",
        );
    }

    /// Test that popping a value from an empty stack fails
    pub fn test_stack_empty_pop_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut stack = Stack::new(writer);

        let res = stack.pop();

        write_test_result(
            writer,
            res.is_err(),
            "shouldn't be able to pop value from empty stack",
        );
    }

    /// Test that pushing into a full stack fails
    pub fn test_stack_push_full_stack_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut stack = Stack::new(writer);

        for i in 0..stack.size() {
            let n = (i % 255) as u8;
            let res = stack.push(n);

            write_test_result(
                writer,
                res.is_ok(),
                "should be able to push value to fill stack",
            );
        }

        let res = stack.push(0_u8);

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "SUCCESS Error pushing a value on the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "FAILURE Success pushing a value on the stack\r").unwrap();
            }
        }
    }

    /// Test that creating a full stack and popping all the values succeeds
    pub fn test_stack_push_full_stack_pop_full_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut stack = Stack::new(writer);

        for i in 0..stack.size() {
            let n = (i % 256) as u8;
            let _res = stack.push(n);
        }

        let res = stack.push(0_u8);

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "SUCCESS Error pushing a value on the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "FAILURE Success pushing a value on the stack\r").unwrap();
            }
        }

        // Now pop all the values
        for i in 0..stack.size() {
            let n = (stack.size() - 1) - (i % 256);
            let res = stack.pop();

            write_test_result(
                writer,
                res.is_ok(),
                "should be able to pop value from filled stack",
            );

            write_test_result(
                writer,
                res.unwrap() == n as u8,
                "popped value from filled stack should equal pushed value",
            );
        }

        let res = stack.pop();

        write_test_result(
            writer,
            res.is_err(),
            "shouldn't be able to pop value from empty stack",
        );
    }
}
