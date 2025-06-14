#![no_std]

use dg_traits::{ClockIn, ClockOut, FloatParameter, IntParameter};
use embassy_time::{Duration, Ticker, Timer};

/// Simple clock forwarder
///
/// With the patch.Init(), this can be useful to wire the B7 push button to one of the output. This
/// way, pulses can be triggered manually for testing purposes.
pub async fn clock_forward(
    mut clock_in: impl ClockIn,
    mut clock_out: impl ClockOut,
    duration: Duration,
) {
    loop {
        clock_in.wait().await;
        clock_out.emit_pulse(duration).await;
    }
}

/// Emits a train of pulse for each incoming clock signal.
pub async fn clock_train(
    mut clock_in: impl ClockIn,
    mut clock_out: impl ClockOut,
    mut pulse_count: impl IntParameter,
    mut pulse_bpm: impl FloatParameter,
) {
    loop {
        clock_in.wait().await;

        let count = pulse_count.get().await;
        let pulse_period_us = ((60.0 * 1_000_000.0) / pulse_bpm.get().await) as u64;

        let pulse_width_us = 10_000.min(pulse_period_us / 2);
        let pulse_width = Duration::from_micros(pulse_width_us);
        let rest_width = Duration::from_micros(pulse_period_us - pulse_width_us);

        for _ in 0..count {
            clock_out.emit_pulse(pulse_width).await;
            Timer::after(rest_width).await;
        }
    }
}

pub async fn clock(mut clock_out: impl ClockOut, mut pulse_pbm: impl FloatParameter) {
    let mut ticker = VaryingTicker::default();

    loop {
        ticker.next(pulse_pbm.get().await).await;
        clock_out.emit_pulse(Duration::from_millis(5)).await;
    }
}

#[derive(Default)]
struct VaryingTicker {
    ticker: Option<Ticker>,
    current_bpm: Option<f32>,
}

impl VaryingTicker {
    pub async fn next(&mut self, bpm: f32) {
        // invalidate ticker if bpm changed
        if self.current_bpm != Some(bpm) {
            self.ticker = None;
            self.current_bpm = Some(bpm);
        }

        self.ticker
            .get_or_insert_with(|| {
                Ticker::every(Duration::from_micros(((60.0 * 1_000_000.0) / bpm) as u64))
            })
            .next()
            .await;
    }
}
