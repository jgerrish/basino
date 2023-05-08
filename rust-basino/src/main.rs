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

    // interrupt::free(|cs| {
    // 	let mut basino_queue_data = unsafe { BASINO_QUEUE_DATA.borrow(cs) };
    // 	basino_queue_data = &Some([0; 4]);
    // });
    // ufmt::uwriteln!(serial, "Got here").unwrap();

    // ufmt::uwriteln!(serial, "queue address: 0x{:X}", unsafe {
    //     core::ptr::addr_of_mut!(rust_basino::BASINO_QUEUE_DATA) as u16
    // },)
    // .unwrap();

    // ufmt::uwriteln!(serial, "stack address: 0x{:X}", unsafe {
    //     core::ptr::addr_of_mut!(rust_basino::BASINO_STACK_BUFFER) as u16
    // },)
    // .unwrap();

    #[cfg(feature = "test-base")]
    rust_basino::tests::run_tests(&mut serial);
    #[cfg(feature = "test-queue")]
    rust_basino::queue::tests::run_tests(&mut serial);
    #[cfg(feature = "test-stack")]
    rust_basino::stack::tests::run_tests(&mut serial);

    loop {
        avr_device::asm::sleep();
    }
}
