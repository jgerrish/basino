//! A crate that has an interpreter for the TinyBASIC Interpretive Language (IL)
//!
//! This crate parses and interprets the TinyBASIC Interpretive
//! Language (IL).  TinyBASIC is implemented as an interpreter in a
//! virtual machine.  This crate provides the virtual machine for that
//! language, called the Interpretive Language in the Tiny BASIC
//! documentation.
//!
//! An IL program is an array of bytes written in the IL bytecode or
//! virtual machine language.
//!
//! This implementation uses a null (value of zero) instruction as an
//! idicator of the end of the IL bytecode stream.
//!
//! The interpreter / virtual machine uses a couple stacks: a
//! computational or expression stack and a control stack.
//!
//! Information from:\
//! [DDJv1](http://archive.org/details/dr_dobbs_journal_vol_01/) Dr. Dobb's Journal - Vol 1 : People's Computer Company\
//! [TBEK](http://www.ittybittycomputers.com/IttyBitty/TinyBasic/TBEK.txt) Tiny BASIC Experimenter's Kit\
//! [Tiny BASIC - Wikipedia](https://en.wikipedia.org/wiki/Tiny_BASIC) Tiny BASIC Wikipedia
#![warn(missing_docs)]

use crate::{basino_il_exec, basino_il_get_next_bytecode, basino_il_init, basino_il_run};

use core::{
    fmt::{Debug, Display, Formatter},
    marker::PhantomData,
};
use ufmt::{uDebug, uWrite, uwrite};

use crate::{Queue, QueueObj, Stack};

/// The kinds of errors that can occur working with the IL
#[derive(Eq, PartialEq)]
pub enum ErrorKind {
    /// A stack overflow occurred
    StackOverflow,
    /// A null pointer was passed in as a parameter or
    /// would have been dereferenced
    NullPointer,
    /// Invalid arguments were passed into a function.
    /// Example includes trying to initialize a virtual machine with
    /// an invalid state.
    InvalidArguments,
    /// The IL bytecode did not correspond to a valid instruction
    InstructionNotFound,
    /// A zero byte was found while loading the next instruction
    /// bytecode
    EndOfProgram,
    /// An attempt was made to run the program while it was stopped
    Stopped,
    /// An unknown error type
    Unknown,
}

impl uDebug for ErrorKind {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        match self {
            ErrorKind::StackOverflow => f.write_str("A stack overflow occurred"),
            ErrorKind::NullPointer => f.write_str("A null pointer was passed in as a parameter"),
            ErrorKind::InvalidArguments => f.write_str("Invalid arguments were passed in"),
            ErrorKind::InstructionNotFound => f.write_str("The instruction was not found"),
            ErrorKind::EndOfProgram => {
                f.write_str("A zero byte was found while loading the next instruction bytecode")
            }
            ErrorKind::Stopped => {
                f.write_str("An attempt was made to run the program while it was stopped")
            }
            ErrorKind::Unknown => f.write_str("An unknown error occurred"),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        match self {
            ErrorKind::StackOverflow => write!(f, "A stack overflow occurred"),
            ErrorKind::NullPointer => write!(f, "A null pointer was passed in as a parameter"),
            ErrorKind::InvalidArguments => write!(f, "Invalid arguments were passed in"),
            ErrorKind::InstructionNotFound => write!(f, "The instruction was not found"),
            ErrorKind::EndOfProgram => write!(
                f,
                "A zero byte was found while loading the next instruction bytecode"
            ),
            ErrorKind::Stopped => write!(
                f,
                "An attempt was made to run the program while it was stopped"
            ),
            ErrorKind::Unknown => write!(f, "An unknown error occurred"),
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        // Use the Display implementation
        write!(f, "{self}")
    }
}

/// An error that can occur when working with an IL system
#[derive(PartialEq)]
pub struct Error {
    /// The kind of the error that occurs, e.g. a stack overflow
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
        write!(f, "{:?}", self.kind)
    }
}

impl uDebug for Error {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        uwrite!(f, "{:?}", self.kind)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// The state of the interpreter
#[repr(u8)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum InterpreterState {
    /// The interpreter is running, not currently executing any
    /// instructions.
    Running = 1,
    /// The interpreter is stopped.
    Stopped = 2,
    /// The instruction is currently executing an instruction.
    /// This means that an instruction bytecode has been read and the
    /// initial one-byte instruction opcode was correctly decoded.
    ExecutingInstruction = 4,
}

impl Debug for InterpreterState {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}, ", *self as u8)?;
        match self {
            InterpreterState::Running => write!(f, "Running"),
            InterpreterState::Stopped => write!(f, "Stopped"),
            InterpreterState::ExecutingInstruction => write!(f, "ExecutingInstruction"),
        }
    }
}

impl uDebug for InterpreterState {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        ufmt::uwrite!(f, "{}, ", *self as u8)?;
        match self {
            InterpreterState::Running => f.write_str("Running"),
            InterpreterState::Stopped => f.write_str("Stopped"),
            InterpreterState::ExecutingInstruction => f.write_str("ExecutingInstruction"),
        }
    }
}

/// The Interpretive Language structure.
/// Holds state about the interpreter and provides access to the input
/// stream and byte code.
#[repr(C)]
pub struct Interpreter<'a> {
    /// The Interpetive Language byte code
    /// Normally, this is going to be stored in the flash memory
    pub byte_code: *const u8,
    /// A pointer to the end of the byte_code array
    /// If the byte code is of length one, and the array is 0-based,
    /// this is the address of entry 1
    pub byte_code_end: *const u8,
    /// Index to the current location in the byte_code
    pub byte_code_ptr: *mut u8,
    /// The state of the interpreter
    pub state: InterpreterState,
    /// The input buffer
    pub queue: *mut QueueObj<'a>,
    /// The stack
    pub stack: *mut Stack<'a>,
    /// We want this structure to last as long as the lifetime of the
    /// array its based on.
    _marker: PhantomData<&'a u8>,
}

impl<'a> Debug for Interpreter<'a> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "Interpreter: byte_code: {:?}, byte_code_ptr: {:?}, byte_code_end: {:?}, state: {:?}, queue: {:?}, stack: {:?}",
            self.byte_code,
            self.byte_code_ptr,
            self.byte_code_end,
            self.state,
            self.queue,
            unsafe { &*self.stack },
        )
    }
}

impl<'a> uDebug for Interpreter<'a> {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        uwrite!(f, "byte_code: {:?}", self.byte_code)?;
        uwrite!(f, ", byte_code_ptr: {:?}", self.byte_code_ptr)?;
        uwrite!(f, ", byte_code_end: {:?}", self.byte_code_end)?;
        uwrite!(f, ", state: {:?}\n", self.state)?;
        uwrite!(f, "  queue: {:?}\n", unsafe { &*self.queue })?;
        uwrite!(f, "  stack: {:?}", unsafe { &*self.stack })
    }
}

impl<'a> Interpreter<'a> {
    /// Create a new interpreter
    ///
    /// # Arguments
    ///
    /// - `byte_code` - A slice to an array of u8 values that contains
    /// the byte code to runner
    ///
    /// - `queue` - The input queue containing keyboard input
    /// - `stack` - The computational stack
    ///
    /// # Returns
    ///
    /// A Result containing the new Interpreter
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::{il::Interpreter, stack::StackImpl, Queue, Stack};
    ///
    /// // Create a new Interpreter
    /// let byte_code_data: [u8; 2] = [0; 2];
    /// let mut input_queue_data: [u8; 16] = [0; 16];
    /// let mut stack_data: [u8; 16] = [0; 16];
    ///
    /// let mut queue = Queue::new(input_queue_data.as_mut_ptr(), input_queue_data.len()).unwrap();
    ///
    /// let mut stack = Stack::new(stack_data.as_mut_ptr(), stack_data.len()).unwrap();
    ///
    /// let _res = Interpreter::new(byte_code_data.as_ptr(), byte_code_data.len(), &mut queue, &mut stack);
    ///
    /// ```
    pub fn new(
        byte_code: *const u8,
        byte_code_len: usize,
        queue: &'a mut Queue<'a>,
        stack: &'a mut Stack<'a>,
    ) -> Result<Self, Error> {
        let mut interpreter = Interpreter {
            byte_code,
            byte_code_end: core::ptr::null_mut::<u8>(),
            byte_code_ptr: core::ptr::null_mut::<u8>(),
            state: InterpreterState::Stopped,
            queue: core::ptr::addr_of_mut!(queue.queue),
            stack: core::ptr::addr_of_mut!(*stack),
            _marker: PhantomData,
        };

        let res = unsafe {
            basino_il_init(
                core::ptr::addr_of_mut!(interpreter),
                byte_code as *const u8,
                byte_code_len as u16,
                core::ptr::addr_of_mut!(queue.queue),
                core::ptr::addr_of_mut!(*stack),
            )
        };

        match res {
            0 => Ok(interpreter),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::InvalidArguments)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    /// Get the next byte code
    ///
    /// # Arguments
    ///
    /// - `self` - The interpreter object
    ///
    /// # Returns
    ///
    /// A Result containing the next byte code
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::{il::Interpreter, stack::StackImpl, Queue, Stack};
    ///
    /// // Create a new Interpreter
    /// let byte_code_data: [u8; 2] = [0x08; 2];
    /// let mut input_queue_data: [u8; 16] = [0; 16];
    /// let mut stack_data: [u8; 16] = [0; 16];
    ///
    /// let mut queue = Queue::new(input_queue_data.as_mut_ptr(), input_queue_data.len()).unwrap();
    ///
    /// let mut stack = Stack::new(stack_data.as_mut_ptr(), stack_data.len()).unwrap();
    ///
    /// let mut interpreter = Interpreter::new(byte_code_data.as_ptr(), byte_code_data.len(), &mut queue, &mut stack).unwrap();
    /// let res = interpreter.get_next_bytecode();
    /// assert!(res.is_ok());
    /// assert_eq!(res.expect("Should get an Ok Result"), 0x08);
    ///
    /// ```
    pub fn get_next_bytecode(&mut self) -> Result<u8, Error> {
        let mut result: u8 = 0;

        let res = unsafe {
            basino_il_get_next_bytecode(
                core::ptr::addr_of_mut!(*self) as *mut Interpreter,
                &mut result,
            )
        };

        match result {
            0 => Ok(res),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            3 => Err(Error::new(ErrorKind::EndOfProgram)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    /// Run the program
    ///
    /// # Arguments
    ///
    /// - `self` - The interpreter object
    ///
    /// # Returns
    ///
    /// A Result indiciating the result of the program run
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::{il::Interpreter, stack::StackImpl, Queue, Stack};
    ///
    /// // Create a new Interpreter
    /// let mut byte_code_data: [u8; 5] = [0; 5];
    /// let mut input_queue_data: [u8; 16] = [0; 16];
    /// let mut stack_data: [u8; 16] = [0; 16];
    ///
    /// byte_code_data[0] = 0x08;
    /// byte_code_data[1] = 0x09;
    /// byte_code_data[2] = 0x76;
    /// byte_code_data[3] = 0x00;
    ///
    /// let mut queue = Queue::new(input_queue_data.as_mut_ptr(), input_queue_data.len()).unwrap();
    ///
    /// let mut stack = Stack::new(stack_data.as_mut_ptr(), stack_data.len()).unwrap();
    ///
    /// let mut interpreter = Interpreter::new(byte_code_data.as_ptr(), byte_code_data.len(), &mut queue, &mut stack).unwrap();
    /// let res = interpreter.run();
    /// assert!(res.is_ok());
    /// // let item_res = stack.pop();
    /// // assert!(item_res.is_ok());
    /// // assert_eq!(item_res.expect("Should get an Ok Result"), 0x76);
    ///
    /// ```
    pub fn run(&mut self) -> Result<(), Error> {
        let res = unsafe { basino_il_run(core::ptr::addr_of_mut!(*self) as *mut Interpreter) };

        match res {
            0 => Ok(()),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::InvalidArguments)),
            3 => Err(Error::new(ErrorKind::InstructionNotFound)),
            4 => Err(Error::new(ErrorKind::EndOfProgram)),
            5 => Err(Error::new(ErrorKind::Stopped)),
            6 => Err(Error::new(ErrorKind::Unknown)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }

    /// Execute an instruction
    ///
    /// # Arguments
    ///
    /// - `self` - The interpreter object
    ///
    /// # Returns
    ///
    /// A Result indiciating the result of the instruction
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::{il::Interpreter, stack::StackImpl, Queue, Stack};
    ///
    /// // Create a new Interpreter
    /// let mut byte_code_data: [u8; 4] = [0; 4];
    /// let mut input_queue_data: [u8; 16] = [0; 16];
    /// let mut stack_data: [u8; 16] = [0; 16];
    ///
    /// byte_code_data[0] = 0x09;
    /// byte_code_data[1] = 0x76;
    /// byte_code_data[2] = 0x00;
    ///
    /// let mut queue = Queue::new(input_queue_data.as_mut_ptr(), input_queue_data.len()).unwrap();
    ///
    /// let mut stack = Stack::new(stack_data.as_mut_ptr(), stack_data.len()).unwrap();
    ///
    /// let mut interpreter = Interpreter::new(byte_code_data.as_ptr(), byte_code_data.len(), &mut queue, &mut stack).unwrap();
    /// let byte_code = interpreter.get_next_bytecode().unwrap();
    /// let res = interpreter.exec(byte_code);
    /// assert!(res.is_ok());
    /// // let item_res = stack.pop();
    /// // assert!(item_res.is_ok());
    /// // assert_eq!(item_res.expect("Should get an Ok Result"), 0x76);
    ///
    /// ```
    pub fn exec(&mut self, opcode: u8) -> Result<(), Error> {
        let res = unsafe { basino_il_exec(self, opcode) };

        match res {
            0 => Ok(()),
            1 => Err(Error::new(ErrorKind::NullPointer)),
            2 => Err(Error::new(ErrorKind::InvalidArguments)),
            3 => Err(Error::new(ErrorKind::InstructionNotFound)),
            4 => Err(Error::new(ErrorKind::Stopped)),
            5 => Err(Error::new(ErrorKind::StackOverflow)),
            6 => Err(Error::new(ErrorKind::Unknown)),
            _ => Err(Error::new(ErrorKind::Unknown)),
        }
    }
}

/// Tests for the Interpretive Language code
pub mod tests {
    use crate::{
        il::{ErrorKind, Interpreter, InterpreterState},
        stack::StackImpl,
        tests::write_test_result,
        Queue, Stack, Usart, BASINO_IL_BYTE_CODE_DATA, BASINO_INPUT_QUEUE_DATA_HANDLE,
        BASINO_STACK_BUFFER_HANDLE,
    };

    use avr_device::interrupt::free;

    /// Run all the tests in this module
    pub fn run_tests(writer: &mut Usart) {
        test_il_init_works(writer);
        test_il_init_zero_length_fails(writer);
        test_il_exec_no_works(writer);
        test_il_exec_lb_works(writer);
        test_il_exec_stopped_works(writer);
        test_il_get_next_bytecode_works(writer);
        test_il_get_next_bytecode_end_of_program_works(writer);
        test_il_get_next_bytecode_end_of_array_works(writer);
        test_il_run_works(writer);
        test_il_run_while_stopped_fails(writer);
        test_il_run_without_eop_works(writer);
    }

    /// Test that initializing the interpreter works
    pub fn test_il_init_works(writer: &mut Usart) {
        let byte_code_data: [u8; 1] = [0; 1];

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let res = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            );

            write_test_result(writer, res.is_ok(), "should initialize interpreter");

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that initializing with a zero length array fails
    pub fn test_il_init_zero_length_fails(writer: &mut Usart) {
        let byte_code_data: [u8; 0] = [0; 0];

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let res = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            );

            write_test_result(
                writer,
                res.is_err(),
                "init with zero length argument should fail to initialize interpreter",
            );
            match res {
                Ok(_) => {
                    write_test_result(
                        writer,
                        false,
                        "init with zero length argument should return InvalidArguments",
                    );
                }
                Err(e) => {
                    write_test_result(
                        writer,
                        e.kind == ErrorKind::InvalidArguments,
                        "init with zero length argument should return InvalidArguments",
                    );
                }
            }
            (queue_handle, stack_handle)
        };
        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that executing a single NO instruction works
    pub fn test_il_exec_no_works(writer: &mut Usart) {
        let byte_code_data: [u8; 2] = [0; 2];

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            let byte_code_ptr_old = interpreter.byte_code_ptr as usize;
            let res = interpreter.exec(0x08);
            let byte_code_ptr_new = interpreter.byte_code_ptr as usize;

            write_test_result(writer, res.is_ok(), "exec should work");

            // Verify the pointer was not advanced
            write_test_result(
                writer,
                byte_code_ptr_old == byte_code_ptr_new,
                "NB exec shouldn't advance the byte code pointer",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that executing Push Literal Byte onto Stack (LB) works
    pub fn test_il_exec_lb_works(writer: &mut Usart) {
        let mut byte_code_data: [u8; 3] = [0; 3];
        byte_code_data[0] = 0x09;
        byte_code_data[1] = 0x76;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            let byte_code = interpreter.get_next_bytecode().unwrap();
            let byte_code_ptr_old = interpreter.byte_code_ptr as usize;
            let res = interpreter.exec(byte_code);
            let byte_code_ptr_new = interpreter.byte_code_ptr as usize;
            assert!(res.is_ok());
            // Test the item item was pushed onto the stack
            let stack = interpreter.stack;
            let res = unsafe { (*stack).pop() };

            write_test_result(writer, res.is_ok(), "LB should pop from stack");
            match res {
                Ok(v) => {
                    write_test_result(writer, v == 0x76, "LB popped value should equal 0x76");
                }
                Err(_e) => {
                    write_test_result(writer, false, "LB popped value should equal 0x76");
                }
            }

            // Verify the pointer was advanced
            write_test_result(
                writer,
                byte_code_ptr_new == byte_code_ptr_old + 1,
                "LB exec should advance the byte code pointer",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that executing while interpreter stops returns the right error code
    pub fn test_il_exec_stopped_works(writer: &mut Usart) {
        let byte_code_data = unsafe { BASINO_IL_BYTE_CODE_DATA.as_mut() };
        byte_code_data[0] = 0x08;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            interpreter.state = InterpreterState::Stopped;

            let res = interpreter.exec(0x08);

            match res {
                Ok(_) => {
                    write_test_result(
                        writer,
                        false,
                        "exec while stopped should return ErrorKind::Stopped",
                    );
                }
                Err(e) => {
                    write_test_result(
                        writer,
                        e.kind == ErrorKind::Stopped,
                        "exec while stopped should return ErrorKind::Stopped",
                    );
                }
            }

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that getting the next bytecode works
    pub fn test_il_get_next_bytecode_works(writer: &mut Usart) {
        let byte_code_data = unsafe { BASINO_IL_BYTE_CODE_DATA.as_mut() };
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x09;
        byte_code_data[2] = 0x76;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            let res = interpreter.get_next_bytecode();

            write_test_result(writer, res.is_ok(), "should get next bytecode");
            match res {
                Ok(bc) => {
                    write_test_result(writer, bc == 0x08, "next bytecode should equal 0x08");
                }
                Err(_) => {
                    write_test_result(writer, false, "next bytecode should equal 0x08");
                }
            }

            let res = interpreter.get_next_bytecode();

            write_test_result(writer, res.is_ok(), "should get next bytecode");
            match res {
                Ok(bc) => {
                    write_test_result(writer, bc == 0x09, "next bytecode should equal 0x09");
                }
                Err(_) => {
                    write_test_result(writer, false, "next bytecode should equal 0x09");
                }
            }

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that getting the next bytecode works.
    /// This tests that an appropriate result and state is set when
    /// end of program is found.
    pub fn test_il_get_next_bytecode_end_of_program_works(writer: &mut Usart) {
        let mut byte_code_data: [u8; 2] = [0; 2];
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x08;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            write_test_result(
                writer,
                interpreter.state == InterpreterState::Running,
                "state should be running",
            );

            let res = interpreter.get_next_bytecode();

            write_test_result(writer, res.is_ok(), "should get next bytecode");
            match res {
                Ok(bc) => {
                    write_test_result(writer, bc == 0x08, "next bytecode should equal 0x08");
                }
                Err(_) => {
                    write_test_result(writer, false, "next bytecode should equal 0x08");
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Running,
                "state should be running",
            );

            // The next bytecode should be a zero byte
            // Since we're not currently executing an instruction, this should fail
            // with an EndOfProgram error.
            let res = interpreter.get_next_bytecode();
            write_test_result(
                writer,
                res.is_err(),
                "get next bytecode should return EndOfProgram",
            );
            match res {
                Ok(_) => {
                    write_test_result(
                        writer,
                        false,
                        "get next bytecode should return EndOfProgram",
                    );
                }
                Err(e) => {
                    write_test_result(
                        writer,
                        e.kind == ErrorKind::EndOfProgram,
                        "get next bytecode should return EndOfProgram",
                    );
                }
            }

            write_test_result(
                writer,
                interpreter.state == InterpreterState::Stopped,
                "state should be stopped",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that getting the next bytecode works.
    /// This tests that an appropriate result and state is set when
    /// end of program is found.
    pub fn test_il_get_next_bytecode_end_of_array_works(writer: &mut Usart) {
        let mut byte_code_data: [u8; 3] = [0; 3];
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x08;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            write_test_result(
                writer,
                interpreter.state == InterpreterState::Running,
                "state should be running",
            );

            let res = interpreter.get_next_bytecode();

            write_test_result(writer, res.is_ok(), "should get next bytecode");
            match res {
                Ok(bc) => {
                    write_test_result(writer, bc == 0x08, "next bytecode should equal 0x08");
                }
                Err(_) => {
                    write_test_result(writer, false, "next bytecode should equal 0x08");
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Running,
                "state should be running",
            );

            // The next bytecode should be another NOP (0x08)
            // This should work.
            let res = interpreter.get_next_bytecode();
            write_test_result(writer, res.is_ok(), "get next bytecode should return Ok");
            match res {
                Ok(_) => {
                    write_test_result(writer, true, "get next bytecode should return Ok");
                }
                Err(_) => {
                    write_test_result(writer, false, "get next bytecode should return Ok");
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Running,
                "state should be running",
            );

            // The pointer should be at the end of the array
            // This should fail.
            let res = interpreter.get_next_bytecode();
            write_test_result(
                writer,
                res.is_err(),
                "get next bytecode should return EndOfProgram",
            );
            match res {
                Ok(_) => {
                    write_test_result(
                        writer,
                        false,
                        "get next bytecode should return EndOfProgram",
                    );
                }
                Err(e) => {
                    write_test_result(
                        writer,
                        e.kind == ErrorKind::EndOfProgram,
                        "get next bytecode should return EndOfProgram",
                    );
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Stopped,
                "state should be stopped",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that running a short simple program works
    pub fn test_il_run_works(writer: &mut Usart) {
        let mut byte_code_data: [u8; 5] = [0; 5];
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x09;
        byte_code_data[2] = 0x76;
        byte_code_data[3] = 0x00;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            let res = interpreter.run();

            write_test_result(writer, res.is_ok(), "should run program");

            // Test the item was pushed onto the stack
            let stack = interpreter.stack;
            let res = unsafe { (*stack).pop() };

            write_test_result(writer, res.is_ok(), "LB should pop from stack");
            match res {
                Ok(v) => {
                    write_test_result(writer, v == 0x76, "LB popped value should equal 0x76");
                }
                Err(_) => {
                    write_test_result(writer, false, "LB popped value should equal 0x76");
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Stopped,
                "successful run should end with a state of stopped",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that running while stopped fails
    pub fn test_il_run_while_stopped_fails(writer: &mut Usart) {
        let mut byte_code_data: [u8; 5] = [0; 5];
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x09;
        byte_code_data[2] = 0x76;
        byte_code_data[3] = 0x00;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            interpreter.run().unwrap();
            let res = interpreter.run();

            match res {
                Ok(_) => {
                    write_test_result(writer, false, "second run should fail");
                }
                Err(e) => {
                    write_test_result(
                        writer,
                        e.kind == ErrorKind::Stopped,
                        "second run should return Stopped",
                    );
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Stopped,
                "successful run should end with a state of stopped",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }

    /// Test that running a program without an end-of-program marker works
    pub fn test_il_run_without_eop_works(writer: &mut Usart) {
        let mut byte_code_data: [u8; 5] = [0; 5];
        byte_code_data[0] = 0x08;
        byte_code_data[1] = 0x09;
        byte_code_data[2] = 0x76;

        let (queue_handle, stack_handle) = {
            let queue_handle = free(|cs| unsafe {
                BASINO_INPUT_QUEUE_DATA_HANDLE
                    .borrow(cs)
                    .replace(None)
                    .unwrap()
            });

            let mut queue = Queue::new(&queue_handle).unwrap();

            let stack_handle =
                free(|cs| unsafe { BASINO_STACK_BUFFER_HANDLE.borrow(cs).replace(None).unwrap() });
            let mut stack = Stack::new_from_array_handle(&stack_handle).unwrap();

            let mut interpreter = Interpreter::new(
                byte_code_data.as_ptr(),
                byte_code_data.len(),
                &mut queue,
                &mut stack,
            )
            .unwrap();

            let res = interpreter.run();

            write_test_result(writer, res.is_ok(), "should run program");

            // Test the item was pushed onto the stack
            let stack = interpreter.stack;
            let res = unsafe { (*stack).pop() };

            write_test_result(writer, res.is_ok(), "LB should pop from stack");
            match res {
                Ok(v) => {
                    write_test_result(writer, v == 0x76, "LB popped value should equal 0x76");
                }
                Err(_) => {
                    write_test_result(writer, false, "LB popped value should equal 0x76");
                }
            }
            write_test_result(
                writer,
                interpreter.state == InterpreterState::Stopped,
                "run without explicit end-of-program should end with a state of stopped",
            );

            (queue_handle, stack_handle)
        };

        free(|cs| unsafe {
            BASINO_INPUT_QUEUE_DATA_HANDLE
                .borrow(cs)
                .replace(Some(queue_handle))
        });
        free(|cs| unsafe {
            BASINO_STACK_BUFFER_HANDLE
                .borrow(cs)
                .replace(Some(stack_handle))
        });
    }
}
