#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Instant};

pub trait ClockIn {
    async fn wait(&mut self) -> Instant;
}

pub trait ClockOut {
    async fn emit_pulse(&mut self, duration: Duration);
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
