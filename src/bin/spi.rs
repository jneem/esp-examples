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

    let mut spi = hal::spi::Spi::new_mosi_only(
        peripherals.SPI2,
        io.pins.gpio7,
        3333u32.kHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let mut delay = Delay::new(&clocks);

    flush(&mut spi);
    loop {
        write_byte(&mut spi, 0x08);
        write_byte(&mut spi, 0x00);
        write_byte(&mut spi, 0x00);
        flush(&mut spi);
        delay.delay_ms(500u32);
        write_byte(&mut spi, 0x00);
        write_byte(&mut spi, 0x08);
        write_byte(&mut spi, 0x00);
        flush(&mut spi);
        delay.delay_ms(500u32);
        write_byte(&mut spi, 0x00);
        write_byte(&mut spi, 0x00);
        write_byte(&mut spi, 0x08);
        flush(&mut spi);
        delay.delay_ms(500u32);
    }
}

fn flush(spi: &mut Spi<SPI2, FullDuplexMode>) {
    for _ in 0..256 {
        nb::block!(spi.send(0)).unwrap();
    }
}

fn write_byte(spi: &mut Spi<SPI2, FullDuplexMode>, mut b: u8) {
    let patterns = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];
    for _ in 0..4 {
        let bits = (b & 0b1100_0000) >> 6;
        nb::block!(spi.send(patterns[bits as usize])).unwrap();
        b <<= 2;
    }
}
