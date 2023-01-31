//! Stack functions and data structures
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
    basino_get_basino_stack_bottom, basino_get_basino_stack_size, basino_get_basino_stack_top,
    basino_get_basino_stack_top_sentinel, basino_stack_init, basino_stack_pop, basino_stack_push,
    BASINO_STACK,
};

/// Basic functions for a stack
pub trait StackImpl {
    /// Initialize the stack
    fn init(&self, writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>)
        -> Result<(), u8>;

    /// Pop a value from the stack
    fn pop(&self) -> Result<u8, u8>;

    /// Push a value onto the stack
    fn push(&self, value: u8) -> Result<(), u8>;
}

impl StackImpl for crate::Stack {
    /// Initialize the stack
    fn init(
        &self,
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) -> Result<(), u8> {
        let res = stack_init_safe(writer);
        ufmt::uwriteln!(writer, "basino_stack_init result: {:?}\r", res).unwrap();
        res?;

        let res = unsafe { basino_get_basino_stack_bottom() };
        ufmt::uwriteln!(writer, "basino_get_basino_stack_bottom result: {}\r", res).unwrap();

        let res = unsafe { basino_get_basino_stack_top() };
        ufmt::uwriteln!(writer, "basino_get_basino_stack_top result: {}\r", res).unwrap();

        let res = unsafe { basino_get_basino_stack_top_sentinel() };
        ufmt::uwriteln!(
            writer,
            "basino_get_basino_stack_top_sentinel result: {}\r",
            res
        )
        .unwrap();

        let res = unsafe { basino_get_basino_stack_size() };
        ufmt::uwriteln!(writer, "basino_get_basino_stack_size result: {}\r", res).unwrap();

        Ok(())
    }

    fn pop(&self) -> Result<u8, u8> {
        let mut result: u8 = 0;
        let res = unsafe { basino_stack_pop(&mut result) };
        match result {
            0 => Ok(res),
            _ => Err(result),
        }
    }

    fn push(&self, value: u8) -> Result<(), u8> {
        let res = unsafe { basino_stack_push(value) };

        match res {
            0 => Ok(()),
            e => Err(e),
        }
    }
}

/// Initialize the stacks
pub fn stack_init_safe(
    writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
) -> Result<(), u8> {
    let stack_top_ptr: *mut u8 = unsafe {
        // Set the stack top address to the address + 1
        // There is now a basino_address_add function that can perform the addition
        // in assembly.  It should be moved there.
        (BASINO_STACK.stack.as_mut_ptr() as usize + BASINO_STACK.stack.len() + 1) as *mut u8
    };

    let stack_size: usize = unsafe { BASINO_STACK.stack.len() };
    let stack_bottom: usize = stack_top_ptr as usize - stack_size;

    ufmt::uwriteln!(writer, "stack_top_ptr: {}\r", stack_top_ptr as usize).unwrap();
    ufmt::uwriteln!(writer, "stack_size: {}\r", stack_size).unwrap();
    ufmt::uwriteln!(writer, "stack_bottom: {}\r", stack_bottom).unwrap();

    let res = unsafe {
        basino_stack_init(
            stack_top_ptr as *mut u8,
            stack_bottom as *mut u8,
            stack_size as u8,
        )
    };

    match res {
        0 => Ok(()),
        r => Err(r),
    }
}

/// A tests module
/// This doesn't use the standard Rust testing framework.  Instead it's a normal
/// public module that can be called by other systems.
pub mod tests {
    use crate::{stack::StackImpl, BASINO_STACK};
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
        test_stack_init_works(writer);
        test_stack_push_works(writer);
        test_stack_empty_pop_fails(writer);
        test_stack_empty_pop_fails(writer);
        // test_stack_push_full_stack_fails(writer);
        test_stack_push_full_stack_pop_full_works(writer);
    }

    /// Test that initializing the stack works
    pub fn test_stack_init_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let stack = unsafe { &BASINO_STACK };

        let res = stack.init(writer);

        match res {
            Err(_) => {
                ufmt::uwriteln!(writer, "FAILURE: Error initializing stack\r").unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "SUCCESS: Initialized stack\r").unwrap();
            }
        }
    }

    /// Test that pushing a value on the stack works
    pub fn test_stack_push_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let stack = unsafe { &BASINO_STACK };

        let res = stack.push(5);

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "FAILURE: Error pushing a value onto the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "SUCCESS: Pushed a value onto the stack\r").unwrap();
            }
        }

        let res = stack.pop();

        match res {
            Err(_) => {
                ufmt::uwriteln!(writer, "FAILURE: Error popping a value from the stack\r").unwrap();
            }
            Ok(v) => {
                ufmt::uwriteln!(writer, "SUCCESS: Popped a value from the stack: {}\r", v).unwrap();
            }
        }
    }

    /// Test that popping a value from an empty stack fails
    pub fn test_stack_empty_pop_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let stack = unsafe { &BASINO_STACK };

        let res = stack.pop();

        match res {
            Err(_) => {
                ufmt::uwriteln!(writer, "SUCCESS: Error popping a value from the stack\r").unwrap();
            }
            Ok(v) => {
                ufmt::uwriteln!(writer, "FAILURE: Popped a value from the stack: {}\r", v).unwrap();
            }
        }
    }

    /// Test that pushing into a full stack fails
    pub fn test_stack_push_full_stack_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let stack = unsafe { &BASINO_STACK };

        for i in 0..stack.stack.len() {
            let n = (i % 256) as u8;
            let res = stack.push(n);

            match res {
                Err(e) => {
                    ufmt::uwriteln!(
                        writer,
                        "FAILURE: Error pushing a value on the stack: {} {}\r",
                        n,
                        e
                    )
                    .unwrap();
                }
                Ok(_) => {
                    ufmt::uwriteln!(
                        writer,
                        "SUCCESS: Success pushing a value on the stack: {}\r",
                        n
                    )
                    .unwrap();
                }
            }
        }

        let res = stack.push(0_u8);

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "SUCCESS: Error pushing a value on the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "FAILURE: Success pushing a value on the stack\r").unwrap();
            }
        }
    }

    /// Test that creating a full stack and popping all the values succeeds
    pub fn test_stack_push_full_stack_pop_full_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let stack = unsafe { &BASINO_STACK };

        for i in 0..stack.stack.len() {
            let n = (i % 256) as u8;
            let res = stack.push(n);

            match res {
                Err(e) => {
                    ufmt::uwriteln!(
                        writer,
                        "FAILURE: Error pushing a value on the stack: {} {}\r",
                        n,
                        e
                    )
                    .unwrap();
                }
                Ok(_) => {
                    ufmt::uwriteln!(
                        writer,
                        "SUCCESS: Success pushing a value on the stack: {}\r",
                        n
                    )
                    .unwrap();
                }
            }
            // assert!(res.is_ok());
        }

        let res = stack.push(0_u8);

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "SUCCESS: Error pushing a value on the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(_) => {
                ufmt::uwriteln!(writer, "FAILURE: Success pushing a value on the stack\r").unwrap();
            }
        }

        // Now pop all the values
        for i in 0..stack.stack.len() {
            let n = 127 - (i % 256) as u8;
            let res = stack.pop();

            match res {
                Err(e) => {
                    ufmt::uwriteln!(
                        writer,
                        "FAILURE: Error pushing a value on the stack: {} {}\r",
                        n,
                        e
                    )
                    .unwrap();
                }
                Ok(value) => {
                    if value == n {
                        ufmt::uwriteln!(
                            writer,
                            "SUCCESS: Success popping a value from the stack: {} == {}\r",
                            value,
                            n
                        )
                        .unwrap();
                    } else {
                        ufmt::uwriteln!(
                            writer,
                            "FAILURE: Failure popping a value from the stack: {} != {}\r",
                            value,
                            n
                        )
                        .unwrap();
                    }
                }
            }
            // assert!(res.is_ok());
        }

        let res = stack.pop();

        match res {
            Err(e) => {
                ufmt::uwriteln!(
                    writer,
                    "SUCCESS: Error popping a value from the stack: {}\r",
                    e
                )
                .unwrap();
            }
            Ok(v) => {
                ufmt::uwriteln!(writer, "FAILURE: Popped a value from the stack: {}\r", v).unwrap();
            }
        }
    }
}
