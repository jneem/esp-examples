#![no_std]
#![no_main]

use embedded_hal::adc::{Channel, OneShot};
use esp_backtrace as _;
use esp_println::println;
use hal::{
    adc::{AdcCalEfuse, AdcConfig, Attenuation, ADC, ADC1},
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

fn read_mean<A, P: Channel<A>>(adc: &mut impl OneShot<A, u16, P>, pin: &mut P) -> u16 {
    (0..64).map(|_| read(adc, pin)).sum::<u16>() / 64
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut delay = Delay::new(&clocks);
    let analog = peripherals.APB_SARADC.split();
    let mut adc_config = AdcConfig::new();
    let atten = Attenuation::Attenuation2p5dB;
    let mut pin = adc_config.enable_pin(io.pins.gpio3.into_analog(), atten);
    let mut adc1 = ADC::<ADC1>::adc(
        &mut system.peripheral_clock_control,
        analog.adc1,
        adc_config,
    )
    .unwrap();

    let init_code = ADC1::get_init_code(atten);
    let cal_mv = ADC1::get_cal_mv(atten);
    let cal_code = ADC1::get_cal_code(atten);

    println!(
        "init code {:?}, cal_mv {}, cal_code {:?}",
        init_code, cal_mv, cal_code
    );

    loop {
        let reading = read_mean(&mut adc1, &mut pin);
        println!("{}", reading);
        delay.delay_ms(1000u32);
    }
}
