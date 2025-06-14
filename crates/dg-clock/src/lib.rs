#![no_std]

use dg_traits::{FloatParameter, GateIn, GateOut, IntParameter};
use embassy_time::{Duration, Timer};

/// Simple clock forwarder
///
/// With the patch.Init(), this can be useful to wire the B7 push button to one of the output. This
/// way, pulses can be triggered manually for testing purposes.
pub async fn clock_forward(
    mut gate_in: impl GateIn,
    mut gate_out: impl GateOut,
    duration: Duration,
) {
    loop {
        gate_in.wait().await;
        gate_out.emit_pulse(duration).await;
    }
}

/// Emits a train of pulse for each incoming gate signal.
pub async fn clock_train(
    mut gate_in: impl GateIn,
    mut gate_out: impl GateOut,
    mut pulse_count: impl IntParameter,
    mut pulse_bpm: impl FloatParameter,
) {
    loop {
        gate_in.wait().await;

        let count = pulse_count.get().await;
        let pulse_period_us = ((60.0 * 1_000_000.0) / pulse_bpm.get().await) as u64;

        let pulse_width_us = 10_000.min(pulse_period_us / 2);
        let pulse_width = Duration::from_micros(pulse_width_us);
        let rest_width = Duration::from_micros(pulse_period_us - pulse_width_us);

        for _ in 0..count {
            gate_out.emit_pulse(pulse_width).await;
            Timer::after(rest_width).await;
        }
    }
}
