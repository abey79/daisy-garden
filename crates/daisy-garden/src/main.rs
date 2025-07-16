#![no_std]
#![no_main]

mod params;

use daisy_embassy::led::UserLed;
use daisy_embassy::new_daisy_board;
use daisy_embassy::pins::{PatchPinC4, PatchPinC5};
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_stm32::spi::Spi;
use embassy_stm32::{
    Config, bind_interrupts,
    exti::ExtiInput,
    gpio::{Level, Output, Pull, Speed},
    peripherals::{self, ADC1, ADC2},
    rcc::{Pll, PllDiv, PllMul, PllPreDiv, PllSource},
    rng::{self, Rng},
    spi,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
use embassy_time::{Duration, Timer};
use fhx::Fhx;
use {defmt_rtt as _, panic_probe as _};

use dg_noise::export::SmallRng;
use dg_noise::{NoiseGenerator, RedNoiseGenerator};

use crate::params::{AdcFloatParameter, AdcIntParameter};

bind_interrupts!(struct Irqs {
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

static FHX_CHANNEL: Channel<CriticalSectionRawMutex, FhxSetMessage, 5> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    config.rcc.pll1 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL50,
        divp: None,
        divq: Some(PllDiv::DIV8), // SPI
        divr: None,
    });
    config.rcc.pll2 = Some(Pll {
        source: PllSource::HSI,
        prediv: PllPreDiv::DIV4,
        mul: PllMul::MUL50,
        divp: Some(PllDiv::DIV8), // 100mhz
        divq: None,
        divr: None,
    });
    config.rcc.hsi48 = Some(Default::default()); // needed for RNG

    let p = embassy_stm32::init(config);

    let daisy_p = new_daisy_board!(p);
    let led = daisy_p.user_led;

    //
    // Setup FHX
    //

    let mut spi_config = spi::Config::default();
    spi_config.frequency = embassy_stm32::time::mhz(1);
    spi_config.miso_pull = Pull::Down; // unused, NC

    let spi = spi::Spi::new_txonly(
        p.SPI2,
        daisy_p.pins.d10,
        daisy_p.pins.d9,
        p.DMA2_CH4,
        spi_config,
    );

    #[unsafe(link_section = ".sram1_bss")]
    static mut TX_BUFFER: [u8; 4] = [0; 4];

    #[expect(static_mut_refs)]
    let fhx = fhx::Fhx::new(
        spi,
        Output::new(daisy_p.pins.d1, Level::High, Speed::Low),
        Output::new(daisy_p.pins.a3, Level::Low, Speed::Low),
        Output::new(daisy_p.pins.a8, Level::Low, Speed::Low),
        Output::new(daisy_p.pins.a9, Level::Low, Speed::Low),
        unsafe { &mut TX_BUFFER },
    );

    spawner
        .spawn(fhx_worker(fhx, FHX_CHANNEL.receiver()))
        .unwrap();

    //
    // Red noise generator
    //

    let mut rng = Rng::new(p.RNG, Irqs);
    let sample_rate = 6;
    let noise_gen = dg_noise::RedNoiseGenerator::new_simple_from_rng(&mut rng, sample_rate);
    spawner
        .spawn(red_noise_gate(
            fhx::CvAddress::Cv1,
            fhx::CvChannel::Channel8,
            fhx::GtAddress::Gt0,
            fhx::GtChannel::Channel8,
            noise_gen,
            sample_rate,
            FHX_CHANNEL.sender(),
        ))
        .unwrap();

    FHX_CHANNEL
        .sender()
        .send(FhxSetMessage::CvPolarity {
            address: fhx::CvAddress::Cv1,
            polarity: 0xFF, // bipolar
        })
        .await;

    //
    // Blinker
    //

    info!("Staring...");
    spawner.spawn(blink(led)).unwrap();

    // // Trigger pulses using the button
    // spawner
    //     .spawn(clock_forward(
    //         ExtiInput::new(daisy_p.pins.b7, p.EXTI8, Pull::Up),
    //         Output::new(daisy_p.pins.c10, Level::Low, Speed::Low),
    //         Duration::from_millis(3),
    //     ))
    //     .unwrap();
    //
    // // Clock train
    // spawner
    //     .spawn(clock_train(
    //         ExtiInput::new(daisy_p.pins.b10, p.EXTI13, Pull::Up),
    //         Output::new(daisy_p.pins.b5, Level::Low, Speed::Low),
    //         AdcIntParameter::new(Adc::new(p.ADC1), daisy_p.pins.c5, 1, 10),
    //         AdcFloatParameter::new(Adc::new(p.ADC2), daisy_p.pins.c4, 80.0, 4000.0, true),
    //     ))
    //     .unwrap();
}

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    loop {
        led.on();
        Timer::after_millis(300).await;

        led.off();
        Timer::after_millis(300).await;
        //info!("Blip");
    }
}

// ---

enum FhxSetMessage {
    CvPolarity {
        address: fhx::CvAddress,
        polarity: u8,
    },

    Cv {
        address: fhx::CvAddress,
        channel: fhx::CvChannel,
        value: u16,
    },

    Gate {
        address: fhx::GtAddress,
        channel: fhx::GtChannel,
        value: bool,
    },
}

#[embassy_executor::task]
async fn fhx_worker(
    mut fhx: Fhx<
        'static,
        Spi<'static, Async>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
    >,
    receiver: Receiver<'static, CriticalSectionRawMutex, FhxSetMessage, 5>,
) {
    loop {
        let msg = receiver.receive().await;

        match msg {
            FhxSetMessage::CvPolarity { address, polarity } => {
                fhx.set_cv_polarity(address, polarity);
            }
            FhxSetMessage::Cv {
                address,
                channel,
                value,
            } => {
                fhx.set_cv_raw(address, channel, value).await;
            }
            FhxSetMessage::Gate {
                address,
                channel,
                value,
            } => {
                if value {
                    fhx.gate_high(address, channel).await;
                } else {
                    fhx.gate_low(address, channel).await;
                }
            }
        }
    }
}

// ---

#[embassy_executor::task]
async fn red_noise_gate(
    address: fhx::CvAddress,
    channel: fhx::CvChannel,
    gate_address: fhx::GtAddress,
    gate_channel: fhx::GtChannel,
    mut noise_generator: RedNoiseGenerator<SmallRng>,
    sampling_rate: u64,
    sender: Sender<'static, CriticalSectionRawMutex, FhxSetMessage, 5>,
) {
    let mut ticker = embassy_time::Ticker::every(Duration::from_hz(sampling_rate));

    loop {
        ticker.next().await;

        let value = noise_generator.sample();

        // Send the value to the FHX worker
        sender
            .send(FhxSetMessage::Cv {
                address,
                channel,
                value,
            })
            .await;

        sender
            .send(FhxSetMessage::Gate {
                address: gate_address,
                channel: gate_channel,
                value: true,
            })
            .await;

        Timer::after_millis(2).await;

        sender
            .send(FhxSetMessage::Gate {
                address: gate_address,
                channel: gate_channel,
                value: false,
            })
            .await;
    }
}

// ---

#[embassy_executor::task(pool_size = 2)]
async fn clock_forward(
    clock_in: ExtiInput<'static>,
    clock_out: Output<'static>,
    duration: Duration,
) {
    dg_clock::clock_forward(clock_in, clock_out, duration).await;
}

#[embassy_executor::task]
async fn clock_train(
    clock_in: ExtiInput<'static>,
    clock_out: Output<'static>,
    pulse_count: AdcIntParameter<'static, ADC1, PatchPinC5>,
    pulse_bpm: AdcFloatParameter<'static, ADC2, PatchPinC4>,
) {
    dg_clock::clock_train(clock_in, clock_out, pulse_count, pulse_bpm).await;
}
