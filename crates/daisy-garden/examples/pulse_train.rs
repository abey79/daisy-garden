#![no_std]
#![no_main]

//! Pulse train generator example
//!
//! ```raw
//!                     ┏━━━━━━━━━━━━━━━━━━━━━━━━━┓
//!                     ┃   .─.             .─.   ┃
//!      train count ───╋─▶(   )           (   )◀─╋─── train rate
//!                     ┃   `─'             `─'   ┃
//!                     ┃                         ┃
//!                     ┃                         ┃
//!                     ┃   .─.             .─.   ┃
//!                     ┃  (   )           (   )  ┃
//!                     ┃   `─'             `─'   ┃
//!                     ┃                         ┃
//!                     ┃            ,       .    ┃
//!       trig pulse ───╋─▶ o        o      ( ) ◀─╋─── pulse out
//!                     ┃            '       '    ┃
//!                     ┃                         ┃
//!                     ┃               ┌─────────╋─── pulse train out
//!                     ┃   .     .     ▼     .   ┃
//! trig pulse train ───╋─▶( )   ( )   ( )   ( )  ┃
//!                     ┃   '     '     '     '   ┃
//!                     ┃                         ┃
//!                     ┃                         ┃
//!                     ┃   .     .     .     .   ┃
//!                     ┃  ( )   ( )   ( )   ( )  ┃
//!                     ┃   '     '     '     '   ┃
//!                     ┃                         ┃
//!                     ┃                         ┃
//!                     ┃   .     .     .     .   ┃
//!                     ┃  ( )   ( )   ( )   ( )  ┃
//!                     ┃   '     '     '     '   ┃
//!                     ┗━━━━━━━━━━━━━━━━━━━━━━━━━┛
//! ```

use daisy_embassy::{
    led::UserLed,
    new_daisy_board,
    pins::{PatchPinC4, PatchPinC5},
};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    Config,
    adc::Adc,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    peripherals::{ADC1, ADC2},
    rcc::{Pll, PllDiv, PllMul, PllPreDiv, PllSource},
};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use daisy_garden::{AdcFloatParameter, AdcIntParameter};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL50,
        divp: Some(PllDiv::DIV8), // ADC1
        divq: None,
        divr: None,
    });
    config.rcc.hsi48 = Some(Default::default()); // needed for RNG

    let p = embassy_stm32::init(config);

    let daisy_p = new_daisy_board!(p);
    let led = daisy_p.user_led;

    //
    // Push button triggers a pulse
    //

    spawner
        .spawn(clock_forward(
            ExtiInput::new(daisy_p.pins.b7, p.EXTI8, Pull::Up),
            Output::new(daisy_p.pins.c10, Level::Low, Speed::Low),
            Duration::from_millis(3),
        ))
        .unwrap();

    //
    // Pulse received in B10 triggers a train of pulses on B5
    //

    spawner
        .spawn(clock_train(
            ExtiInput::new(daisy_p.pins.b10, p.EXTI13, Pull::Up),
            Output::new(daisy_p.pins.b5, Level::Low, Speed::Low),
            AdcIntParameter::new(Adc::new(p.ADC1), daisy_p.pins.c5, 1, 10),
            AdcFloatParameter::new(Adc::new(p.ADC2), daisy_p.pins.c4, 80.0, 4000.0, true),
        ))
        .unwrap();

    //
    // Blinker
    //

    info!("Staring...");
    spawner.spawn(blink(led)).unwrap();
}

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    loop {
        led.on();
        Timer::after_millis(300).await;

        led.off();
        Timer::after_millis(300).await;
        //info!("Blip");
    }
}

// ---

#[embassy_executor::task(pool_size = 2)]
async fn clock_forward(
    clock_in: ExtiInput<'static>,
    clock_out: Output<'static>,
    duration: Duration,
) {
    dg_clock::clock_forward(clock_in, dg_types::Pin(clock_out), duration).await;
}

#[embassy_executor::task]
async fn clock_train(
    clock_in: ExtiInput<'static>,
    clock_out: Output<'static>,
    pulse_count: AdcIntParameter<'static, ADC1, PatchPinC5>,
    pulse_bpm: AdcFloatParameter<'static, ADC2, PatchPinC4>,
) {
    dg_clock::clock_train(clock_in, dg_types::Pin(clock_out), pulse_count, pulse_bpm).await;
}
