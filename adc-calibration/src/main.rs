#![no_std]
#![no_main]

use embedded_hal::adc::{Channel, OneShot};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    adc::{AdcCalLine, AdcConfig, Attenuation, ADC, ADC1},
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    Delay, IO,
};

fn read<A, P>(adc: &mut impl OneShot<A, u16, P>, pin: &mut P) -> u16
where
    P: Channel<A>,
{
    let mut sum = 0;
    let mut min = u16::MAX;
    let mut max = u16::MIN;

    for _ in 0..10 {
        let result = nb::block!(adc.read(pin)).ok().unwrap();
        min = min.min(result);
        max = max.max(result);
        sum += result;
    }
    sum -= min;
    sum -= max;
    sum / 8
}

fn read_mean<A, P: Channel<A>>(adc: &mut impl OneShot<A, u16, P>, pin: &mut P) -> u32 {
    ((0..64).map(|_| read(adc, pin) as u32).sum::<u32>() / 64)
        * Attenuation::Attenuation2p5dB.ref_mv() as u32
        / 4096
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut delay = Delay::new(&clocks);
    let analog = peripherals.APB_SARADC.split();
    let mut adc_config = AdcConfig::new();

    let mut pin2p5 = adc_config.enable_pin_with_cal::<_, AdcCalLine<_>>(
        io.pins.gpio4.into_analog(),
        Attenuation::Attenuation2p5dB,
    );
    let mut pin6 = adc_config.enable_pin_with_cal::<_, AdcCalLine<_>>(
        io.pins.gpio0.into_analog(),
        Attenuation::Attenuation6dB,
    );
    let mut adc1 = ADC::<ADC1>::adc(analog.adc1, adc_config).unwrap();

    loop {
        let reading = read_mean(&mut adc1, &mut pin2p5);
        println!("Atten 2.5: {}", reading);
        delay.delay_ms(100u32);
        let reading = read_mean(&mut adc1, &mut pin6);
        println!("Atten 6: {}", reading);
        delay.delay_ms(1000u32);
    }
}
