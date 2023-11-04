#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::sync::atomic;

use defmt_rtt as _;
use hal::{clock::ClockControl, peripherals, prelude::*, Delay};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    critical_section::with(|_cs| {
        defmt::error!("{}", defmt::Display2Format(info));
    });

    loop {
        atomic::compiler_fence(atomic::Ordering::SeqCst);
    }
}

#[entry]
fn main() -> ! {
    let peripherals = peripherals::Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    defmt::println!("Hello ESP32-C3!");

    for i in 0..30 {
        defmt::println!("{}, ah ha ha", i);
        delay.delay_ms(1000u32);
    }
    panic!("done");
}
