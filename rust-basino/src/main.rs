//! A simple stack implementation for Arduino devices
#![warn(missing_docs)]
#![no_std]
#![no_main]

// use ufmt::UnstableDoAsFormatter;
// use ufmt_utils::WriteAdapter;

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    // rust_basino::stack::tests::run_tests(&mut serial);
    rust_basino::queue::tests::run_tests(&mut serial);

    loop {
        avr_device::asm::sleep();
    }
}
