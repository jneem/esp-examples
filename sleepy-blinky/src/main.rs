#![no_std]
#![no_main]

use core::time::Duration;

use esp_backtrace as _;
use hal::{
    clock::ClockControl,
    gpio::RTCPinWithResistors,
    peripherals::Peripherals,
    prelude::*,
    rmt::{Channel0, PulseCode, TxChannel, TxChannelConfig, TxChannelCreator},
    rtc_cntl::sleep::{RtcioWakeupSource, TimerWakeupSource, WakeupLevel},
    Delay, Rmt, Rtc, IO,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Initialize the RMT peripheral.
    let rmt = Rmt::new(
        peripherals.RMT,
        // 80 MHz is the default RMT clock frequency, according to:
        // https://github.com/esp-rs/esp-hal/blob/0c47ceda3afbc71dc2f540589811257eab51199f/esp-hal-common/src/soc/esp32c3/mod.rs#L28
        // It appears that arbitrary frequencies are not supported here; only integer divisors of the base frequency:
        // https://github.com/esp-rs/esp-hal/blob/0c47ceda3afbc71dc2f540589811257eab51199f/esp-hal-common/src/rmt.rs#L234
        80u32.MHz(),
        &mut system.peripheral_clock_control,
        &clocks,
    )
    .unwrap();

    let mut delay = Delay::new(&clocks);

    let config = TxChannelConfig {
        clk_divider: 1,
        ..TxChannelConfig::default()
    };

    // Our LED is hooked up to pin 7.
    let mut channel = rmt.channel0.configure(io.pins.gpio7, config).unwrap();

    let wakeup_pins: &mut [(&mut dyn RTCPinWithResistors, WakeupLevel)] = &mut [
        (&mut io.pins.gpio2, WakeupLevel::Low),
        (&mut io.pins.gpio3, WakeupLevel::High),
    ];
    let rtcio = RtcioWakeupSource::new(wakeup_pins);
    let timer = TimerWakeupSource::new(Duration::from_secs(10));

    channel = color(channel, 64, 0, 0);
    delay.delay_ms(500u32);
    channel = color(channel, 0, 64, 0);
    delay.delay_ms(500u32);
    channel = color(channel, 0, 0, 64);
    delay.delay_ms(500u32);
    color(channel, 0, 0, 0);
    rtc.sleep_deep(&[&timer, &rtcio], &mut delay);
}

// The WS2812 spec sheet has two different durations.
// The shorter duration is 0.40µs. That's 32 cycles at 80MHz.
const SHORT: u16 = 32;
// The shorter duration is 0.85µs. That's 68 cycles at 80MHz.
const LONG: u16 = 68;

// We send a "one" bit by setting the pin high for a long time and low for
// a short time.
const ONE: PulseCode = PulseCode {
    level1: true,
    length1: LONG,
    level2: false,
    length2: SHORT,
};

// We send a "zero" bit by setting the pin high for a short time and low for
// a long time.
const ZERO: PulseCode = PulseCode {
    level1: true,
    length1: SHORT,
    level2: false,
    length2: LONG,
};

// We send a "reset" code by setting the pin low for 50µs. That's 4000 cycles
// at 80MHz.
const RESET: PulseCode = PulseCode {
    level1: false,
    length1: 0,
    level2: false,
    length2: 4000,
};

// Tell the led to change color.
fn color<const CH: u8>(ch: Channel0<CH>, r: u8, g: u8, b: u8) -> Channel0<CH> {
    // We need to send 25 pulses: 24 bits and the reset code.
    let mut buf = [0u32; 25];

    // According to the spec sheet, the order of bytes is GRB.
    write_byte(&mut buf[0..8], g);
    write_byte(&mut buf[8..16], r);
    write_byte(&mut buf[16..24], b);
    buf[24] = RESET.into();

    ch.transmit(&buf).wait().unwrap()
}

// Convert a byte into a pulse code. Store the result in the buffer `out`, which
// must be 8 bytes long.
fn write_byte(out: &mut [u32], mut b: u8) {
    let one: u32 = ONE.into();
    let zero: u32 = ZERO.into();

    for sig in out {
        // Highest order bits get sent first.
        let bit = b & 0b1000_0000;
        *sig = if bit != 0 { one } else { zero };
        b <<= 1;
    }
}
