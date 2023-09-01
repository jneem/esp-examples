#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};
use embedded_hal_async::spi::SpiBus;
use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    dma::DmaPriority,
    gdma::{Channel0, Gdma},
    peripherals::{Peripherals, SPI2},
    prelude::*,
    spi::{dma::SpiDma, FullDuplexMode, SpiMode},
    timer::TimerGroup,
    Rtc, IO,
};

macro_rules! singleton {
    ($val:expr, $T:ty) => {{
        static STATIC_CELL: ::static_cell::StaticCell<$T> = ::static_cell::StaticCell::new();
        STATIC_CELL.init($val)
    }};
}

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

    hal::embassy::init(
        &clocks,
        hal::systimer::SystemTimer::new(peripherals.SYSTIMER),
    );
    hal::interrupt::enable(
        hal::peripherals::Interrupt::DMA_CH0,
        hal::interrupt::Priority::Priority1,
    )
    .unwrap();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let descriptors = singleton!([0u32; 8 * 3], [u32; 8 * 3]);
    let dma = Gdma::new(peripherals.DMA, &mut system.peripheral_clock_control);
    let dma_channel = dma
        .channel0
        .configure(false, descriptors, &mut [], DmaPriority::Priority0);

    let spi = hal::spi::Spi::new_mosi_only(
        peripherals.SPI2,
        io.pins.gpio7,
        3333u32.kHz(),
        SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .with_dma(dma_channel);

    let executor = singleton!(Executor::new(), Executor);
    executor.run(|spawner| {
        spawner.spawn(led_task(spi)).ok();
    });
}

#[embassy_executor::task]
async fn led_task(mut spi: SpiDma<'static, SPI2, Channel0, FullDuplexMode>) {
    let mut buf = [0; 48];
    loop {
        write_color(&mut buf, 16, 0, 0);
        SpiBus::write(&mut spi, &buf).await.unwrap();
        Timer::after(Duration::from_millis(500)).await;
        write_color(&mut buf, 0, 16, 0);
        SpiBus::write(&mut spi, &buf).await.unwrap();
        Timer::after(Duration::from_millis(500)).await;
        write_color(&mut buf, 0, 0, 16);
        SpiBus::write(&mut spi, &buf).await.unwrap();
        Timer::after(Duration::from_millis(500)).await;
    }
}

fn write_color(buf: &mut [u8; 48], r: u8, g: u8, b: u8) {
    write_byte(&mut buf[0..4], g);
    write_byte(&mut buf[4..8], r);
    write_byte(&mut buf[8..12], b);
}

fn write_byte(buf: &mut [u8], mut b: u8) {
    let patterns = [0b1000_1000, 0b1000_1110, 0b1110_1000, 0b1110_1110];
    for out in buf {
        let bits = (b & 0b1100_0000) >> 6;
        *out = patterns[bits as usize];
        b <<= 2;
    }
}
