#![no_std]

pub async fn clock_forward(
    mut gate_in: impl dg_traits::GateIn,
    mut gate_out: impl dg_traits::GateOut,
    duration: embassy_time::Duration,
) {
    loop {
        gate_in.wait().await;
        gate_out.emit_pulse(duration).await;
    }
}
