#![no_std]
#![allow(async_fn_in_trait)]

use embassy_time::{Duration, Instant};

pub trait GateIn {
    async fn wait(&mut self) -> Instant;
}

pub trait GateOut {
    async fn emit_pulse(&mut self, duration: Duration);
}
