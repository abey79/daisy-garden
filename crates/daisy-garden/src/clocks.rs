use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_time::{Duration, Instant, Timer};

use dg_traits::{ClockIn, ClockOut};

pub struct ExtiInputClockIn<'a>(ExtiInput<'a>);

impl<'a> ExtiInputClockIn<'a> {
    pub fn new(input: ExtiInput<'a>) -> Self {
        Self(input)
    }
}

impl ClockIn for ExtiInputClockIn<'_> {
    async fn wait(&mut self) -> Instant {
        self.0.wait_for_falling_edge().await;
        Instant::now()
    }
}

pub struct OutputClockOut<'a>(Output<'a>);

impl<'a> OutputClockOut<'a> {
    pub fn new(output: Output<'a>) -> Self {
        Self(output)
    }
}

impl ClockOut for OutputClockOut<'_> {
    async fn emit_pulse(&mut self, duration: Duration) {
        self.0.set_high();
        Timer::after(duration).await;
        self.0.set_low();
    }
}
