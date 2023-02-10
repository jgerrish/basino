//! Test the stack implementation for Arduino devices
#![warn(missing_docs)]
#![no_std]
#![no_main]

use panic_halt as _;

use rust_basino::stack::tests::run_tests;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    run_tests(&mut serial);

    loop {
        avr_device::asm::sleep();
    }
}
