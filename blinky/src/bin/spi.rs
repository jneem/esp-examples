#![no_std]
#![no_main]

use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    peripherals::{Peripherals, SPI2},
    prelude::*,
    spi::{FullDuplexMode, Spi, SpiMode},
    timer::TimerGroup,
    Delay, Rtc, IO,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(
        peripherals.TIMG0,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(
        peripherals.TIMG1,
        &clocks,
        &mut system.peripheral_clock_control,
    );
    let mut wdt1 = timer_group1.wdt;

    rtc.swd.disable();
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Initialize the SPI peripheral. MOSI ("main out sub in") means that we're
    // transmitting data to the peripheral, and not expecting a response.
    let mut spi = hal::spi::Spi::new_mosi_only(
        peripherals.SPI2,
        io.pins.gpio7,
        // 3.333 MHz means that each cycle takes 300ns.
        3333u32.kHz(),
        // The SPI mode affects how the signal is synchronized with
        // the clock. It doesn't affect the duration of the high and low
        // parts of the signal, so it doesn't actually matter which mode
        // we use.
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut delay = Delay::new(&clocks);

    loop {
        write_color(&mut spi, 5, 0, 0);
        delay.delay_ms(500u32);
        write_color(&mut spi, 0, 5, 0);
        delay.delay_ms(500u32);
        write_color(&mut spi, 0, 0, 5);
        delay.delay_ms(500u32);
    }
}

fn write_color(spi: &mut Spi<SPI2, FullDuplexMode>, r: u8, g: u8, b: u8) {
    // According to the spec sheet, the order of bytes is GRB.
    write_byte(spi, g);
    write_byte(spi, r);
    write_byte(spi, b);

    // We need to send "0" for 50 µs. Each byte is 8 cycles and each cycle is 0.3 µs,
    // so 21 bytes is enough.
    for _ in 0..21 {
        nb::block!(spi.send(0)).unwrap();
    }
}

// Write out a single byte in the format expected by the WS2812.
fn write_byte(spi: &mut Spi<SPI2, FullDuplexMode>, mut b: u8) {
    let patterns = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];
    for _ in 0..4 {
        let bits = (b & 0b1100_0000) >> 6;
        nb::block!(spi.send(patterns[bits as usize])).unwrap();
        b <<= 2;
    }
}
