#![no_std]
#![no_main]

use daisy_embassy::led::UserLed;
use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_time::{Duration, Instant, Timer};

use dg_traits::{GateIn, GateOut};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    info!("Hello World!");
    let daisy_p = new_daisy_board!(p);
    let led = daisy_p.user_led;

    spawner.spawn(blink(led)).unwrap();
    spawner
        .spawn(clock_forward(
            ExtiInput::new(daisy_p.pins.b10, p.EXTI13, Pull::Up),
            Output::new(daisy_p.pins.b5, Level::Low, Speed::Low),
            Duration::from_millis(7),
        ))
        .unwrap();

    spawner
        .spawn(clock_forward(
            ExtiInput::new(daisy_p.pins.b7, p.EXTI8, Pull::Up),
            Output::new(daisy_p.pins.c10, Level::Low, Speed::Low),
            Duration::from_millis(3),
        ))
        .unwrap();
}

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    loop {
        info!("on");
        led.on();
        Timer::after_millis(300).await;

        info!("off");
        led.off();
        Timer::after_millis(300).await;
    }
}

#[embassy_executor::task(pool_size = 2)]
async fn clock_forward(gate_in: ExtiInput<'static>, gate_out: Output<'static>, duration: Duration) {
    dg_clock::clock_forward(ExtiInputGateIn(gate_in), OutputGateOut(gate_out), duration).await;
}

struct ExtiInputGateIn<'a>(ExtiInput<'a>);

impl GateIn for ExtiInputGateIn<'_> {
    async fn wait(&mut self) -> Instant {
        self.0.wait_for_falling_edge().await;
        Instant::now()
    }
}

struct OutputGateOut<'a>(Output<'a>);

impl GateOut for OutputGateOut<'_> {
    async fn emit_pulse(&mut self, duration: Duration) {
        self.0.set_high();
        Timer::after(duration).await;
        self.0.set_low();
    }
}
