use daisy_embassy::led::UserLed;
use daisy_embassy::new_daisy_board;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Pull, Speed},
    rcc::{Pll, PllDiv, PllMul, PllPreDiv, PllSource},
    {Config, bind_interrupts, peripherals, rng, spi},
};
use embassy_time::Timer;

use crate::{FhxCv, FhxGate};

bind_interrupts!(struct Irqs {
    HASH_RNG => rng::InterruptHandler<peripherals::RNG>;
});

#[allow(non_snake_case)]
pub struct PatchInit {
    pub cv_1: daisy_embassy::pins::PatchPinC5,
    pub cv_2: daisy_embassy::pins::PatchPinC4,
    pub cv_3: daisy_embassy::pins::PatchPinC3,
    pub cv_4: daisy_embassy::pins::PatchPinC2,
    pub cv_5: daisy_embassy::pins::PatchPinC6,
    pub cv_6: daisy_embassy::pins::PatchPinC7,
    pub cv_7: daisy_embassy::pins::PatchPinC8,
    pub cv_8: daisy_embassy::pins::PatchPinC9,

    pub b7: daisy_embassy::pins::PatchPinB7,
    pub b8: daisy_embassy::pins::PatchPinB8,

    pub cv_out_1: daisy_embassy::pins::PatchPinC10,

    pub gate_in_1: daisy_embassy::pins::PatchPinB10,
    pub gate_in_2: daisy_embassy::pins::PatchPinB9,
    pub gate_out_1: daisy_embassy::pins::PatchPinB5,
    pub gate_out_2: daisy_embassy::pins::PatchPinB6,

    pub rng: rng::Rng<'static, peripherals::RNG>,

    pub EXTI0: peripherals::EXTI0,
    pub EXTI1: peripherals::EXTI1,
    pub EXTI2: peripherals::EXTI2,
    pub EXTI3: peripherals::EXTI3,
    pub EXTI4: peripherals::EXTI4,
    pub EXTI5: peripherals::EXTI5,
    pub EXTI6: peripherals::EXTI6,
    pub EXTI7: peripherals::EXTI7,
    pub EXTI8: peripherals::EXTI8,
    pub EXTI9: peripherals::EXTI9,
    pub EXTI10: peripherals::EXTI10,
    pub EXTI11: peripherals::EXTI11,
    pub EXTI12: peripherals::EXTI12,
    pub EXTI13: peripherals::EXTI13,
    pub EXTI14: peripherals::EXTI14,
    pub EXTI15: peripherals::EXTI15,

    pub ADC1: peripherals::ADC1,
    pub ADC2: peripherals::ADC2,
    pub ADC3: peripherals::ADC3,
}

impl PatchInit {
    pub fn new(spawner: &Spawner) -> Self {
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
            divp: Some(PllDiv::DIV8), // ADC1
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

        spawner.spawn(crate::fhx::fhx_worker(fhx)).unwrap();

        //
        // Blinker
        //

        info!("Staring...");
        spawner.spawn(blink(led)).unwrap();

        PatchInit {
            cv_1: daisy_p.pins.c5,
            cv_2: daisy_p.pins.c4,
            cv_3: daisy_p.pins.c3,
            cv_4: daisy_p.pins.c2,
            cv_5: daisy_p.pins.c6,
            cv_6: daisy_p.pins.c7,
            cv_7: daisy_p.pins.c8,
            cv_8: daisy_p.pins.c9,

            b7: daisy_p.pins.b7,
            b8: daisy_p.pins.b8,

            cv_out_1: daisy_p.pins.c10,

            gate_in_1: daisy_p.pins.b10,
            gate_in_2: daisy_p.pins.b9,
            gate_out_1: daisy_p.pins.b5,
            gate_out_2: daisy_p.pins.b6,

            rng: rng::Rng::new(p.RNG, Irqs),

            EXTI0: p.EXTI0,
            EXTI1: p.EXTI1,
            EXTI2: p.EXTI2,
            EXTI3: p.EXTI3,
            EXTI4: p.EXTI4,
            EXTI5: p.EXTI5,
            EXTI6: p.EXTI6,
            EXTI7: p.EXTI7,
            EXTI8: p.EXTI8,
            EXTI9: p.EXTI9,
            EXTI10: p.EXTI10,
            EXTI11: p.EXTI11,
            EXTI12: p.EXTI12,
            EXTI13: p.EXTI13,
            EXTI14: p.EXTI14,
            EXTI15: p.EXTI15,

            ADC1: p.ADC1,
            ADC2: p.ADC2,
            ADC3: p.ADC3,
        }
    }

    pub fn fhx_cv(&self, address: fhx::CvAddress, channel: fhx::CvChannel) -> FhxCv {
        FhxCv::new(address, channel)
    }

    pub fn fhx_gate(&self, address: fhx::GtAddress, channel: fhx::GtChannel) -> FhxGate {
        FhxGate::new(address, channel)
    }
}

#[embassy_executor::task]
async fn blink(mut led: UserLed<'static>) {
    loop {
        led.on();
        Timer::after_millis(300).await;

        led.off();
        Timer::after_millis(300).await;
    }
}
