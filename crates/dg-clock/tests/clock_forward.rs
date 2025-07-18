use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::pin::pin;

use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Timer};

use dg_types::{ClockIn, ClockOut};

#[derive(Debug, Clone)]
pub struct Pulse {
    time: Instant,
    duration: Duration,
}

impl Pulse {
    pub fn new(time: Instant, duration: Duration) -> Self {
        Self { time, duration }
    }

    pub fn time(&self) -> Instant {
        self.time
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn assert_shortly_after(&self, other: Instant) {
        assert!(self.time >= other, "{} is not after {}", self.time, other);
        assert!(
            self.time <= other + Duration::from_millis(3),
            "{} is not before {}",
            self.time,
            other + Duration::from_millis(3)
        );
    }
}

impl From<(Instant, Duration)> for Pulse {
    fn from((time, duration): (Instant, Duration)) -> Self {
        Self::new(time, duration)
    }
}

#[derive(Debug, Clone)]
pub struct MockClockIn {
    events: BinaryHeap<Reverse<Instant>>,
}

impl MockClockIn {
    pub fn new(events: impl IntoIterator<Item = Instant>) -> Self {
        MockClockIn {
            events: events.into_iter().map(Reverse).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl ClockIn for MockClockIn {
    async fn wait(&mut self) -> Instant {
        let now = Instant::now();
        while let Some(Reverse(next_event)) = self.events.peek() {
            if *next_event <= now {
                self.events.pop();
            } else {
                break;
            }
        }

        if let Some(next_event) = self.events.pop() {
            Timer::at(next_event.0).await;
            next_event.0
        } else {
            // wait forever if no events are left
            std::future::pending().await
        }
    }
}

#[derive(Debug)]
pub struct MockClockOut<'a> {
    pulses: &'a mut Vec<Pulse>,
}

impl<'a> MockClockOut<'a> {
    pub fn new(pulses: &'a mut Vec<Pulse>) -> Self {
        Self { pulses }
    }
}

impl ClockOut for MockClockOut<'_> {
    async fn emit_pulse(&mut self, duration: Duration) {
        let now = Instant::now();
        self.pulses.push(Pulse::new(now, duration));
        Timer::after(duration).await;
    }
}

#[tokio::test]
async fn test_mock_clock_in() {
    let now = Instant::now();
    let mut clock_in = MockClockIn::new([
        now + Duration::from_millis(10),
        now + Duration::from_millis(20),
        now + Duration::from_millis(30),
    ]);

    assert_eq!(clock_in.wait().await, now + Duration::from_millis(10));
    assert_eq!(clock_in.wait().await, now + Duration::from_millis(20));
    assert_eq!(clock_in.wait().await, now + Duration::from_millis(30));
}

#[tokio::test]
async fn test_mock_clock_in_drops_past_event() {
    let now = Instant::now();
    let mut clock_in = MockClockIn::new([
        now + Duration::from_millis(10),
        now + Duration::from_millis(20),
        now + Duration::from_millis(30),
    ]);

    assert_eq!(clock_in.wait().await, now + Duration::from_millis(10));
    Timer::after(Duration::from_millis(15)).await;
    assert_eq!(clock_in.wait().await, now + Duration::from_millis(30));
    assert!(clock_in.is_empty());
}

#[tokio::test]
async fn test_clock_forward() {
    let now = Instant::now();
    let mut pulses = Vec::new();

    {
        let mut clock_forward_mut = pin!(dg_clock::clock_forward(
            MockClockIn::new([
                now + Duration::from_millis(10),
                now + Duration::from_millis(20),
            ]),
            MockClockOut::new(&mut pulses),
            Duration::from_millis(5),
        ));

        let mut end_fut = pin!(async {
            Timer::after(Duration::from_millis(50)).await;
        });

        loop {
            match select(&mut clock_forward_mut, &mut end_fut).await {
                Either::First(_) => {}
                Either::Second(_) => break,
            }
        }
    }

    assert_eq!(pulses.len(), 2);
    pulses[0].assert_shortly_after(now + Duration::from_millis(10));
    assert_eq!(pulses[0].duration, Duration::from_millis(5));

    pulses[1].assert_shortly_after(now + Duration::from_millis(20));
    assert_eq!(pulses[1].duration, Duration::from_millis(5));
}

#[tokio::test]
async fn test_clock_forward_drops_pulse() {
    let now = Instant::now();
    let mut pulses = Vec::new();

    {
        let mut clock_forward_mut = pin!(dg_clock::clock_forward(
            MockClockIn::new([
                now + Duration::from_millis(10),
                now + Duration::from_millis(20),
                now + Duration::from_millis(30),
            ]),
            MockClockOut::new(&mut pulses),
            Duration::from_millis(15),
        ));

        let mut end_fut = pin!(async {
            Timer::after(Duration::from_millis(50)).await;
        });

        loop {
            match select(&mut clock_forward_mut, &mut end_fut).await {
                Either::First(_) => {}
                Either::Second(_) => break,
            }
        }
    }

    assert_eq!(pulses.len(), 2);
    pulses[0].assert_shortly_after(now + Duration::from_millis(10));
    assert_eq!(pulses[0].duration, Duration::from_millis(15));

    pulses[1].assert_shortly_after(now + Duration::from_millis(30));
    assert_eq!(pulses[1].duration, Duration::from_millis(15));
}
