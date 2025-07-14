#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Instant};
use embedded_hal::digital::OutputPin;
use embedded_hal_async::digital::Wait;

pub trait ClockIn {
    async fn wait(&mut self) -> Instant;
}

impl<T: Wait> ClockIn for T {
    async fn wait(&mut self) -> Instant {
        self.wait_for_rising_edge().await.unwrap();
        Instant::now()
    }
}

pub trait ClockOut {
    async fn emit_pulse(&mut self, duration: Duration);
}

impl<T: OutputPin> ClockOut for T {
    async fn emit_pulse(&mut self, duration: Duration) {
        self.set_high().unwrap();
        embassy_time::Timer::after(duration).await;
        self.set_low().unwrap();
    }
}

pub trait IntParameter {
    async fn get(&mut self) -> i32;
}

impl IntParameter for i32 {
    async fn get(&mut self) -> i32 {
        *self
    }
}

pub trait FloatParameter {
    async fn get(&mut self) -> f32;
}

impl FloatParameter for f32 {
    async fn get(&mut self) -> f32 {
        *self
    }
}
