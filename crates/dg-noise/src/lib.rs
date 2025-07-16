#![no_std]

use rand::rngs::SmallRng;
use rand_core::{RngCore, SeedableRng};

pub mod export {
    pub use rand::rngs::SmallRng;
}

pub trait NoiseGenerator {
    /// Generate a white noise sample
    fn sample(&mut self) -> u16;
}

pub struct WhiteNoiseGenerator<R: RngCore> {
    rng: R,
}

impl WhiteNoiseGenerator<SmallRng> {
    pub fn new_simple_from_rng(seed_rng: &mut impl RngCore) -> Self {
        let rng = SmallRng::from_rng(seed_rng).expect("Failed to create SmallRng from seed");
        Self::new(rng)
    }
}

impl<R: RngCore> WhiteNoiseGenerator<R> {
    pub fn new(rng: R) -> Self {
        Self { rng }
    }
}

impl<R: RngCore> NoiseGenerator for WhiteNoiseGenerator<R> {
    fn sample(&mut self) -> u16 {
        (self.rng.next_u64() >> 16) as u16
    }
}

// ---

pub struct RedNoiseGenerator<R: RngCore> {
    white_noise: WhiteNoiseGenerator<R>,
    accumulator: f64,
    sample_rate: u64,
}

impl RedNoiseGenerator<SmallRng> {
    pub fn new_simple_from_rng(seed_rng: &mut impl RngCore, sample_rate: u64) -> Self {
        let white_noise = WhiteNoiseGenerator::new_simple_from_rng(seed_rng);
        Self::new(white_noise, sample_rate)
    }
}

impl<R: RngCore> RedNoiseGenerator<R> {
    pub fn new(white_noise: WhiteNoiseGenerator<R>, sample_rate: u64) -> Self {
        Self {
            white_noise,
            accumulator: 0.0,
            sample_rate,
        }
    }
}

impl<R: RngCore> NoiseGenerator for RedNoiseGenerator<R> {
    fn sample(&mut self) -> u16 {
        // Get white noise sample and convert to signed floating point
        let white_sample = (self.white_noise.sample() as f64 - 32768.0) / 32768.0;

        // Integration with frequency-dependent scaling
        // The scaling factor ensures proper red noise characteristics
        let scale = 1.0 / libm::sqrt(self.sample_rate as f64);
        self.accumulator += white_sample * scale;

        // Apply high-pass filter to remove DC drift
        // This prevents the accumulator from wandering too far
        let hp_cutoff = 1.0 / (self.sample_rate as f64); // 1Hz cutoff
        self.accumulator *= 1.0 - hp_cutoff;

        // Convert back to u16 range with soft clipping
        let output = libm::tanh(self.accumulator) * 32767.0 + 32768.0;
        output.clamp(0.0, 65535.0) as u16
    }
}
