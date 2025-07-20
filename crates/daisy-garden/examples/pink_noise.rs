#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

use daisy_garden::{FhxCv, FhxGate, PatchInit};
use dg_noise::export::SmallRng;
use dg_noise::{NoiseGenerator, RedNoiseGenerator};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut patch_init = PatchInit::new(&spawner);

    let sample_rate = 6;
    let noise_gen =
        dg_noise::RedNoiseGenerator::new_simple_from_rng(&mut patch_init.rng, sample_rate);

    spawner
        .spawn(red_noise_gate(
            patch_init.fhx_gate(fhx::GtAddress::Gt0, fhx::GtChannel::Channel8),
            patch_init.fhx_cv(fhx::CvAddress::Cv1, fhx::CvChannel::Channel8),
            noise_gen,
            sample_rate,
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn red_noise_gate(
    gate: FhxGate, //TODO: should use clock in?
    cv_out: FhxCv,
    mut noise_generator: RedNoiseGenerator<SmallRng>,
    sampling_rate: u64,
) {
    let mut ticker = embassy_time::Ticker::every(Duration::from_hz(sampling_rate));

    loop {
        ticker.next().await;

        let value = noise_generator.sample();
        cv_out.set_value(value).await;
        gate.set_high().await;
        Timer::after_millis(2).await;
        gate.set_low().await;
    }
}
