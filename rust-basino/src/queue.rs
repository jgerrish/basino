//! Queue functions and data structures
#![warn(missing_docs)]

use crate::{
    basino_queue_get, basino_queue_get_head, basino_queue_get_last_head,
    basino_queue_get_queue_end, basino_queue_get_queue_start, basino_queue_get_tail,
    basino_queue_init, basino_queue_put, Queue, QueueObj,
};

use core::fmt::{Debug, Display, Formatter};
use ufmt::{uDebug, uWrite};

/// The kinds of errors that can occur working with queues
#[derive(Eq, PartialEq)]
pub enum ErrorKind {
    /// Queue is empty
    QueueEmpty,
    /// Queue is full
    QueueFull,
    /// A null pointer was passed in as a parameter or
    /// would have been dereferenced
    NullPointer,
    /// Invalid arguments were passed into a function.
    /// Example includes trying to initialize a queue with the start
    /// greater than the end.
    InvalidArguments,
    /// An unknown error type
    Unknown,
}

impl uDebug for ErrorKind {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        match self {
            ErrorKind::QueueEmpty => f.write_str("The queue is empty"),
            ErrorKind::QueueFull => f.write_str("The queue is full"),
            ErrorKind::NullPointer => f.write_str("A null pointer was passed in as a parameter"),
            ErrorKind::InvalidArguments => f.write_str("Invalid arguments were passed in"),
            ErrorKind::Unknown => f.write_str("An unknown error occurred"),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            ErrorKind::QueueEmpty => write!(f, "The queue is empty"),
            ErrorKind::QueueFull => write!(f, "The queue is full"),
            ErrorKind::NullPointer => write!(f, "A null pointer was passed in as a parameter"),
            ErrorKind::InvalidArguments => write!(f, "Invalid arguments were passed in"),
            ErrorKind::Unknown => write!(f, "An unknown error occurred"),
        }
    }
}

/// An error that can occur when working with a temperature server
#[derive(PartialEq)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Create a new Error with a given ErrorKind variant
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// Basic functions for a queue
pub trait QueueImpl {
    /// Put or enqueue a value in the queue
    fn put(&mut self, value: u8) -> Result<(), Error>;

    /// Get or dequeue a value from the queue
    fn get(&mut self) -> Result<u8, Error>;

    // Debugging functions

    /// Get the start of the queue
    fn get_start(&mut self) -> Result<*const u8, Error>;
    /// Get the end of the queue
    fn get_end(&mut self) -> Result<*const u8, Error>;

    /// Get the current head of the queue
    fn get_head(&mut self) -> Result<*const u8, Error>;
    /// Get the last head of the queue
    fn get_last_head(&mut self) -> Result<*const u8, Error>;
    /// Get the current tail of the queue
    fn get_tail(&mut self) -> Result<*const u8, Error>;
}

impl Queue {
    fn new(queue_array: &mut [u8]) -> Result<Self, Error> {
        // Initialize the queue
        // Set the queue start to the beginning of the queue array
        let queue_start = queue_array.as_mut_ptr();

        // Set the queue end to the start plus the length minus one
        // Why minus one?  Not because of the head/tail limitations,
        // but because start and end are pointers and for an array of length one
        // they should point to the same element.
        let queue_end = (queue_start as usize + queue_array.len() - 1) as *mut u8;

        let mut queue = Self {
            queue: QueueObj {
                queue: core::ptr::null_mut::<u8>(),
                start: core::ptr::null_mut::<u8>(),
                end: core::ptr::null_mut::<u8>(),
                head: core::ptr::null_mut::<u8>(),
                last_head: core::ptr::null_mut::<u8>(),
                tail: core::ptr::null_mut::<u8>(),
            },
            queue_len: queue_array.len(),
        };

        let res = unsafe {
            basino_queue_init(
                core::ptr::addr_of_mut!(queue.queue) as *mut QueueObj,
                queue_start as *mut u8,
                queue_end as *mut u8,
            )
        };

        match res {
            0 => Ok(queue),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::InvalidArguments)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }
}

impl QueueImpl for Queue {
    fn put(&mut self, value: u8) -> Result<(), Error> {
        let result = unsafe {
            basino_queue_put(core::ptr::addr_of_mut!(self.queue) as *mut QueueObj, value)
        };

        match result {
            0 => Ok(()),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::QueueFull)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    fn get(&mut self) -> Result<u8, Error> {
        let mut result: u8 = 0;

        let res = unsafe {
            basino_queue_get(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                &mut result,
            )
        };

        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::QueueEmpty)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    // Debugging functions

    fn get_start(&mut self) -> Result<*const u8, Error> {
        let mut result: u8 = 0;
        let res = unsafe {
            basino_queue_get_queue_start(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };
        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    fn get_end(&mut self) -> Result<*const u8, Error> {
        let mut result: u8 = 0;
        let res = unsafe {
            basino_queue_get_queue_end(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };
        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    fn get_head(&mut self) -> Result<*const u8, Error> {
        let mut result: u8 = 0;
        let res = unsafe {
            basino_queue_get_head(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };
        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    fn get_last_head(&mut self) -> Result<*const u8, Error> {
        let mut result: u8 = 0;
        let res = unsafe {
            basino_queue_get_last_head(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };
        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    fn get_tail(&mut self) -> Result<*const u8, Error> {
        let mut result: u8 = 0;
        let res = unsafe {
            basino_queue_get_tail(
                core::ptr::addr_of_mut!(self.queue) as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };
        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }
}

/// A tests module
/// This doesn't use the standard Rust testing framework.  Instead it's a normal
/// public module that can be called by other systems.
pub mod tests {
    use crate::{
        queue::{
            basino_queue_get, basino_queue_get_head, basino_queue_get_last_head,
            basino_queue_get_queue_end, basino_queue_get_queue_start, basino_queue_get_tail,
            basino_queue_init, basino_queue_put, ErrorKind, Queue, QueueImpl, QueueObj,
        },
        tests::write_test_result,
        BASINO_QUEUE_DATA,
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
        test_queue_init_works(writer);
        test_queue_empty_get_fails(writer);
        test_queue_put_works(writer);
        test_queue_put_twice_works(writer);
        test_queue_put_fill_works(writer);
        test_queue_put_and_get_fill_works(writer);
        test_queue_head_wraps_works(writer);
        test_queue_head_wraps_nonfilled_works(writer);
        test_queue_head_wraps_nonemptied_works(writer);
        test_queue_last_head_update(writer);
        test_queue_init_null_queue_fails(writer);
        test_queue_basino_queue_put_null_queue_fails(writer);
        test_queue_basino_queue_get_null_queue_fails(writer);
        test_queue_basino_queue_get_last_head_null_queue_fails(writer);
        test_queue_basino_queue_get_head_null_queue_fails(writer);
        test_queue_basino_queue_get_tail_null_queue_fails(writer);
        test_queue_basino_queue_get_queue_start_null_queue_fails(writer);
        test_queue_basino_queue_get_queue_end_null_queue_fails(writer);
        test_queue_basino_queue_get_last_head_works(writer);
        test_queue_basino_queue_get_head_works(writer);
        test_queue_basino_queue_get_tail_works(writer);
        test_queue_basino_queue_get_queue_start_works(writer);
        test_queue_basino_queue_get_queue_end_works(writer);
    }

    /// Test that initializing the queue works
    pub fn test_queue_init_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let res = Queue::new(basino_queue_data);

        write_test_result(writer, res.is_ok(), "should initialize queue");
    }

    /// Test that getting from an empty queue fails
    pub fn test_queue_empty_get_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let res = queue.get();
        write_test_result(writer, res.is_err(), "get from empty queue should fail");
    }

    /// Test that putting an item into the queue works
    pub fn test_queue_put_works(writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

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

    /// Test that putting two items and getting two items from the queue works
    /// This puts both items first, then gets both items.
    pub fn test_queue_put_twice_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        // First, put both items

        let res = queue.put(5);

        write_test_result(writer, res.is_ok(), "should put 5 into queue");
        let res = queue.put(3);

        write_test_result(writer, res.is_ok(), "should put 3 into queue");

        // Now get both items

        let res = queue.get();

        match res {
            Ok(v) => {
                write_test_result(writer, v == 5, "get should return 5");
            }
            Err(_e) => {
                write_test_result(writer, false, "get should be ok and return 5");
            }
        }
        let res = queue.get();

        match res {
            Ok(v) => {
                write_test_result(writer, v == 3, "get should return 3");
            }
            Err(_e) => {
                write_test_result(writer, false, "get should be ok and return 3");
            }
        }
    }

    /// Test that filling the queue works
    pub fn test_queue_put_fill_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        for i in 1..queue.queue_len {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        let res = queue.put(130_u8);
        write_test_result(writer, res.is_err(), "last put to full queue should fail");
        match res {
            Err(e) => {
                write_test_result(
                    writer,
                    e.kind == ErrorKind::QueueFull,
                    "last put should fail with QueueFull error",
                );
            }
            _ => {
                write_test_result(writer, false, "last put should fail with QueueFull error");
            }
        }
    }

    /// Test that filling the queue and getting all the values works
    pub fn test_queue_put_and_get_fill_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        for i in 1..queue.queue_len {
            let _res = queue.put((i % 256) as u8);
        }

        for i in 1..queue.queue_len {
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
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let res = queue.put(0x23_u8);
        write_test_result(writer, res.is_ok(), "single put of 0x23 should work");

        let res = queue.get();
        write_test_result(writer, res.is_ok(), "single get should work");
        write_test_result(writer, res.unwrap() == 0x23, "single get should equal 0x23");

        for i in 1..queue.queue_len {
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
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        // ufmt::uwriteln!(writer, "array: {:?}", unsafe { BASINO_QUEUE_DATA }).unwrap();
        // for i in 1..queue.queue_len goes from 1 to (queue.queue_len - 1) inclusive
        // So, if queue.queue_len is 4, this iterates through [1, 2, 3]
        for i in 1..queue.queue_len {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        for i in 1..queue.queue_len {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == ((i % 256) as u8),
                "should be able to get filled values ",
            );
        }

        for i in 1..=2 {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 2 more values",
            );
        }

        for i in 1..=2 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == (i as u8),
                "should be able to get values",
            );
        }
    }

    /// Test a case where the head wraps around
    /// This tests a case where we don't fill the queue all the way,
    /// then read those values, then try to wrap
    pub fn test_queue_head_wraps_nonfilled_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        // for i in 1..queue.queue_len goes from 1 to (queue.queue_len - 1) inclusive
        // So, if queue.queue_len is 4, this iterates through [1, 2, 3]
        for i in 1..=2 {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        for i in 1..=2 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == ((i % 256) as u8),
                "should be able to get filled values ",
            );
        }

        for i in 1..=2 {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 2 more values",
            );
        }

        for i in 1..=2 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == (i as u8),
                "should be able to get values",
            );
        }

        // The queue should now be empty
        let res = queue.get();
        write_test_result(writer, res.is_err(), "get from empty queue should fail");
    }

    /// Test where we wrap the tail and head with gets in between filling the queue
    /// Don't empty the queue all the way when getting values before the wrap
    pub fn test_queue_head_wraps_nonemptied_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        // put in two items [1, 2]
        for i in [1, 2] {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to put in two items");
        }

        // Remove one item
        let res = queue.get();
        write_test_result(
            writer,
            res.unwrap() == 1_u8,
            "should be able to get one item ",
        );

        // Put in two items, [3, 4]
        for i in [3, 4] {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 2 more values",
            );
        }

        // Get three items
        for i in [2, 3, 4] {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == (i as u8),
                "should be able to get all values",
            );
        }

        // The queue should now be empty
        let res = queue.get();
        write_test_result(writer, res.is_err(), "get from empty queue should fail");
    }

    /// Test a case where the last head wasn't being updated in the
    /// end-of-queue code path
    pub fn test_queue_last_head_update(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        // for i in 1..queue.queue_len goes from 1 to (queue.queue_len - 1) inclusive
        // So, if queue.queue_len is 4, this iterates through [1, 2, 3]
        for i in 1..=2 {
            let res = queue.put((i % 256) as u8);
            write_test_result(writer, res.is_ok(), "should be able to fill queue");
        }

        for i in 1..=2 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == ((i % 256) as u8),
                "should be able to get filled values ",
            );
        }

        for i in 1..=2 {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 2 more values",
            );
        }

        for i in 1..=2 {
            let res = queue.get();
            write_test_result(
                writer,
                res.unwrap() == (i as u8),
                "should be able to get values",
            );
        }

        // The queue should now be empty
        let res = queue.get();
        write_test_result(writer, res.is_err(), "get from empty queue should fail");

        for i in 1..=3 {
            let res = queue.put(i as u8);
            write_test_result(
                writer,
                res.is_ok(),
                "should be able to put in 3 more values",
            );
        }
    }

    /// Test that init with a NULL queue pointer fails
    pub fn test_queue_init_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start_ptr = unsafe { core::ptr::addr_of_mut!(BASINO_QUEUE_DATA) as *mut u8 };
        let len = unsafe { BASINO_QUEUE_DATA.len() };

        let queue_end_ptr: *mut u8 = (queue_start_ptr as usize + len - 1) as *mut u8;

        let res = unsafe {
            basino_queue_init(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                queue_start_ptr as *mut u8,
                queue_end_ptr as *mut u8,
            )
        };

        write_test_result(writer, res == 1, "init should fail with null queue pointer");
    }

    /// Test that get with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "get should fail with null queue pointer",
        );
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_put_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let res = unsafe { basino_queue_put(core::ptr::null_mut::<u16>() as *mut QueueObj, 5) };

        write_test_result(writer, res == 1, "put should fail with null queue pointer");
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_last_head_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get_last_head(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "put should fail with null queue pointer",
        );
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_head_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get_head(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "put should fail with null queue pointer",
        );
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_tail_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get_tail(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "put should fail with null queue pointer",
        );
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_queue_start_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get_queue_start(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "put should fail with null queue pointer",
        );
    }

    /// Test that put with a NULL queue pointer fails
    /// Tests the raw error code
    pub fn test_queue_basino_queue_get_queue_end_null_queue_fails(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let mut result: u8 = 0;
        let _res = unsafe {
            basino_queue_get_queue_end(
                core::ptr::null_mut::<u16>() as *mut QueueObj,
                core::ptr::addr_of_mut!(result),
            )
        };

        write_test_result(
            writer,
            result == 1,
            "put should fail with null queue pointer",
        );
    }

    // Test the debugging functions

    /// Test that get_last_head works
    pub fn test_queue_basino_queue_get_last_head_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start = unsafe { BASINO_QUEUE_DATA.as_mut_ptr() };
        let queue_end = (queue_start as usize + unsafe { BASINO_QUEUE_DATA.len() } - 1) as *mut u8;
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let last_head = queue.get_last_head();

        write_test_result(
            writer,
            last_head.unwrap() == queue_end,
            "get_last_head should return correct value",
        );
    }

    /// Test that get_last_head works
    pub fn test_queue_basino_queue_get_head_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start = unsafe { BASINO_QUEUE_DATA.as_mut_ptr() };
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let head = queue.get_head();

        write_test_result(
            writer,
            head.unwrap() == queue_start,
            "get_head should return correct value",
        );
    }

    /// Test that get_last_head works
    pub fn test_queue_basino_queue_get_tail_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start = unsafe { BASINO_QUEUE_DATA.as_mut_ptr() };
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let tail = queue.get_tail();

        write_test_result(
            writer,
            tail.unwrap() == queue_start,
            "get_tail should return correct value",
        );
    }

    /// Test that get_last_head works
    pub fn test_queue_basino_queue_get_queue_start_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start = unsafe { BASINO_QUEUE_DATA.as_mut_ptr() };
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let res = queue.get_start();

        write_test_result(
            writer,
            res.unwrap() == queue_start,
            "get_start should return correct value",
        );
    }

    /// Test that get_queue_end works
    pub fn test_queue_basino_queue_get_queue_end_works(
        writer: &mut Usart<USART0, Pin<Input, PD0>, Pin<Output, PD1>>,
    ) {
        let queue_start = unsafe { BASINO_QUEUE_DATA.as_mut_ptr() };
        let queue_end = (queue_start as usize + unsafe { BASINO_QUEUE_DATA.len() } - 1) as *mut u8;
        let basino_queue_data = unsafe { BASINO_QUEUE_DATA.as_mut() };
        let mut queue = Queue::new(basino_queue_data).unwrap();

        let res = queue.get_end();

        write_test_result(
            writer,
            res.unwrap() == queue_end,
            "get_end should return correct value",
        );
    }
}
