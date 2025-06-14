use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Output;
use embassy_time::{Duration, Instant, Timer};

use dg_traits::{GateIn, GateOut};

pub struct ExtiInputGateIn<'a>(ExtiInput<'a>);

impl<'a> ExtiInputGateIn<'a> {
    pub fn new(input: ExtiInput<'a>) -> Self {
        Self(input)
    }
}

impl GateIn for ExtiInputGateIn<'_> {
    async fn wait(&mut self) -> Instant {
        self.0.wait_for_falling_edge().await;
        Instant::now()
    }
}

pub struct OutputGateOut<'a>(Output<'a>);

impl<'a> OutputGateOut<'a> {
    pub fn new(output: Output<'a>) -> Self {
        Self(output)
    }
}

impl GateOut for OutputGateOut<'_> {
    async fn emit_pulse(&mut self, duration: Duration) {
        self.0.set_high();
        Timer::after(duration).await;
        self.0.set_low();
    }
}
