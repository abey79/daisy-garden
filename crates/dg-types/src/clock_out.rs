use embassy_time::Duration;
use embedded_hal::digital::OutputPin;

pub trait ClockOut {
    async fn emit_pulse(&mut self, duration: Duration);
}

/// Newtype wrapper for a pin to implement `ClockOut`.
pub struct Pin<T>(pub T);

impl<T: OutputPin> ClockOut for Pin<T> {
    async fn emit_pulse(&mut self, duration: Duration) {
        self.0.set_high().unwrap();
        embassy_time::Timer::after(duration).await;
        self.0.set_low().unwrap();
    }
}

impl<T0, T1> ClockOut for (T0, T1)
where
    T0: ClockOut,
    T1: ClockOut,
{
    async fn emit_pulse(&mut self, duration: Duration) {
        ::embassy_futures::join::join(self.0.emit_pulse(duration), self.1.emit_pulse(duration))
            .await;
    }
}

impl<T0, T1, T2> ClockOut for (T0, T1, T2)
where
    T0: ClockOut,
    T1: ClockOut,
    T2: ClockOut,
{
    async fn emit_pulse(&mut self, duration: Duration) {
        ::embassy_futures::join::join3(
            self.0.emit_pulse(duration),
            self.1.emit_pulse(duration),
            self.2.emit_pulse(duration),
        )
        .await;
    }
}
