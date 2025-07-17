#![no_std]
#![allow(async_fn_in_trait)]

mod clock_in;
mod clock_out;
mod float_parameter;
mod int_parameter;

pub use self::{
    clock_in::ClockIn,
    clock_out::{ClockOut, Pin},
    float_parameter::FloatParameter,
    int_parameter::IntParameter,
};
