#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use hal::{clock::ClockControl, peripherals, prelude::*, Delay};

#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    println!("Hello ESP32-C3!");

    for i in 0..30 {
        println!("{}, ah ha ha", i);
        delay.delay_ms(1000u32);
    }
    panic!("done");
}
