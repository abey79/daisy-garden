#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Instant};

pub trait TimeSource {
    fn now(&self) -> Instant;
    async fn sleep(&self, duration: Duration);
    async fn sleep_until(&self, instant: Instant);
}

pub trait GateIn {
    async fn wait(&mut self) -> Instant;
}

pub trait GateOut {
    async fn emit_pulse(&mut self, duration: Duration);
}
