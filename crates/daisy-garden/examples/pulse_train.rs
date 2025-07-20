#![no_std]
#![no_main]

use daisy_embassy::pins::{PatchPinC4, PatchPinC5};
use embassy_executor::Spawner;
use embassy_stm32::{
    adc::Adc,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    peripherals::{ADC1, ADC2},
};
use embassy_time::Duration;
use {defmt_rtt as _, panic_probe as _};

use daisy_garden::{AdcFloatParameter, AdcIntParameter, PatchInit};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let patch_init = PatchInit::new(&spawner);

    //
    // Push button triggers a pulse
    //

    spawner
        .spawn(clock_forward(
            ExtiInput::new(patch_init.b7, patch_init.EXTI8, Pull::Up),
            //Note: CV out can also be used as a gate out....
            Output::new(patch_init.cv_out_1, Level::Low, Speed::Low),
            Duration::from_millis(3),
        ))
        .unwrap();

    //
    // Pulse received in B10 triggers a train of pulses on B5
    //

    spawner
        .spawn(clock_train(
            ExtiInput::new(patch_init.gate_in_1, patch_init.EXTI13, Pull::Up),
            Output::new(patch_init.gate_out_1, Level::Low, Speed::Low),
            AdcIntParameter::new(Adc::new(patch_init.ADC1), patch_init.cv_1, 1, 10),
            AdcFloatParameter::new(
                Adc::new(patch_init.ADC2),
                patch_init.cv_2,
                80.0,
                4000.0,
                true,
            ),
        ))
        .unwrap();
}

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
