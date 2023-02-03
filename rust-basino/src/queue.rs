//! Queue functions and data structures
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
    basino_queue_get, basino_queue_get_head, basino_queue_get_last_head,
    basino_queue_get_queue_end, basino_queue_get_queue_start, basino_queue_get_tail,
    basino_queue_init, basino_queue_put, BASINO_QUEUE,
};

/// Basic functions for a queue
pub trait QueueImpl {
    /// Initialize the stack
    fn init(&self, writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>)
        -> Result<(), u8>;

    /// Put or enqueue a value in the queue
    fn put(&self, value: u8) -> Result<(), u8>;

    /// Get or dequeue a value from the queue
    fn get(&self) -> Result<u8, u8>;

    // Debugging functions

    /// Get the start of the queue
    fn get_start(&self) -> *const u16;
    /// Get the end of the queue
    fn get_end(&self) -> *const u16;

    /// Get the current head of the queue
    fn get_head(&self) -> *const u16;
    /// Get the last head of the queue
    fn get_last_head(&self) -> *const u16;
    /// Get the current tail of the queue
    fn get_tail(&self) -> *const u16;
}

impl QueueImpl for crate::Queue {
    fn init(
        &self,
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) -> Result<(), u8> {
        let res = queue_init_safe(writer);
        res?;

        Ok(())
    }

    fn put(&self, value: u8) -> Result<(), u8> {
        let result = unsafe { basino_queue_put(value) };

        match result {
            0 => Ok(()),
            e => Err(e),
        }
    }

    fn get(&self) -> Result<u8, u8> {
        let mut result: u8 = 0;

        let res = unsafe { basino_queue_get(&mut result) };

        match result {
            0 => Ok(res),
            e => Err(e),
        }
    }

    // Debugging functions

    fn get_start(&self) -> *const u16 {
        unsafe { basino_queue_get_queue_start() }
    }

    fn get_end(&self) -> *const u16 {
        unsafe { basino_queue_get_queue_end() }
    }

    fn get_head(&self) -> *const u16 {
        unsafe { basino_queue_get_head() }
    }

    fn get_last_head(&self) -> *const u16 {
        unsafe { basino_queue_get_last_head() }
    }

    fn get_tail(&self) -> *const u16 {
        unsafe { basino_queue_get_tail() }
    }
}

/// Initialize the stacks
pub fn queue_init_safe(
    _writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
) -> Result<(), u8> {
    // Set the queue start to the beginning of the queue array
    let queue_start = unsafe { BASINO_QUEUE.queue.as_mut_ptr() };

    // Set the queue end to the start plus the length minus one
    // Why minus one?
    // Simple example: queue length of one, then the start and end
    // address should be the same.  If we don't subtract one, the end
    // would be beyond the length of the queue.
    let queue_end = unsafe { (queue_start as usize + BASINO_QUEUE.queue.len() - 1) as *mut u8 };

    let res = unsafe { basino_queue_init(queue_start as *mut u8, queue_end as *mut u8) };

    match res {
        0 => Ok(()),
        r => Err(r),
    }
}

/// A tests module
/// This doesn't use the standard Rust testing framework.  Instead it's a normal
/// public module that can be called by other systems.
pub mod tests {
    use crate::{queue::QueueImpl, tests::write_test_result, BASINO_QUEUE};
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
        test_queue_init_works(writer);
        test_queue_empty_get_fails(writer);
        test_queue_put_works(writer);
        test_queue_put_fill_works(writer);
        test_queue_put_and_get_fill_works(writer);
        test_queue_head_wraps_works(writer);
    }

    /// Test that initializing the queue works
    pub fn test_queue_init_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let queue = unsafe { &BASINO_QUEUE };
        let res = queue.init(writer);
        write_test_result(writer, res.is_ok(), "should initialize queue");
    }

    /// Test that getting from an empty queue fails
    pub fn test_queue_empty_get_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);
        let res = queue.get();
        write_test_result(writer, res.is_err(), "get from empty queue should fail");
    }

    /// Test that initializing the queue works
    pub fn test_queue_put_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);

        let res = queue.put(5);
        write_test_result(writer, res.is_ok(), "should put 5 into queue");

        let res = queue.get();

        match res {
            Ok(v) => {
                write_test_result(writer, true, "get should be ok");
                write_test_result(writer, v == 5, "get should return 5");
            }
            Err(_e) => {
                write_test_result(writer, false, "get should be ok and return 5");
            }
        }
    }

    /// Test that filling the queue works
    pub fn test_queue_put_fill_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);

        for i in 1..queue.queue.len() {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        let res = queue.put(130_u8);
        write_test_result(writer, res.is_err(), "last put to full queue should fail");
    }

    /// Test that filling the queue and getting all the values works
    pub fn test_queue_put_and_get_fill_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);

        for i in 1..queue.queue.len() {
            let _res = queue.put((i % 256) as u8);
        }

        for i in 1..queue.queue.len() {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == ((i % 256) as u8),
                "should get filled value",
            );
        }
    }

    /// Test that putting a value, getting it, and then filling the queue works
    /// This tests for the case where we move the head and tail
    pub fn test_queue_put_get_put_fill_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);

        let res = queue.put(0x23_u8);
        write_test_result(writer, res.is_ok(), "single put of 0x23 should work");

        let res = queue.get();
        write_test_result(writer, res.is_ok(), "single get should work");
        write_test_result(writer, res.unwrap() == 0x23, "single get should equal 0x23");

        for i in 1..queue.queue.len() {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        let res = queue.put(130_u8);
        write_test_result(writer, res.is_err(), "last put to full queue should fail");
    }

    /// Test a case where the head wraps around
    pub fn test_queue_head_wraps_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue = unsafe { &BASINO_QUEUE };
        let _res = queue.init(writer);

        for i in 1..queue.queue.len() {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        for i in 1..queue.queue.len() {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == ((i % 256) as u8),
                "should be able to get filled values ",
            );
        }

        for i in 1..=10 {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 10 more values",
            );
        }

        for i in 1..=10 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == (i as u8),
                "should be able to get values",
            );
        }
    }
}
