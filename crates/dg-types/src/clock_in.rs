use embassy_time::Instant;
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
