use defmt::info;
use embassy_stm32::adc::{Adc, AdcChannel, Instance};

use dg_traits::IntParameter;

pub struct AdcIntParameter<'d, T, P>
where
    T: Instance,
{
    adc: Adc<'d, T>,
    pin: P,

    min: i32,
    max: i32,
}

impl<'d, T, P> AdcIntParameter<'d, T, P>
where
    T: Instance,
    P: AdcChannel<T> + 'd,
{
    pub fn new(adc: Adc<'d, T>, pin: P, min: i32, max: i32) -> Self {
        assert!(min <= max, "min must be less than max");
        Self { adc, pin, min, max }
    }
}

impl<'d, T, P> IntParameter for AdcIntParameter<'d, T, P>
where
    T: Instance,
    P: AdcChannel<T> + 'd,
{
    async fn get(&mut self) -> i32 {
        let Self { adc, pin, min, max } = self;

        //  0                  2**16
        //  │                    │
        //  ┌───┬───┬───┬───┬───┐
        //  │ L │ … │ … │ … │ H │
        //  └───┴───┴───┴───┴───┘
        //  │                    │
        //  0                  H-L+1
        //

        let value = adc.blocking_read(pin) as i64;

        info!("ADC value: {}", value);

        // Correct for patch.Init pots, which are inverted and return 0-2**15
        //TODO: make that optional?
        let value = (32768 - value) << 1;

        let min = *min as i64;
        let max = *max as i64;
        let range = max - min + 1;

        let res = min + range * value / 65536;

        res as i32
    }
}

pub struct AdcFloatParameter<'d, T, P>
where
    T: Instance,
{
    adc: Adc<'d, T>,
    pin: P,

    min: f32,
    max: f32,

    log_scale: bool,
}

impl<'d, T, P> AdcFloatParameter<'d, T, P>
where
    T: Instance,
    P: AdcChannel<T> + 'd,
{
    pub fn new(adc: Adc<'d, T>, pin: P, mut min: f32, mut max: f32, log_scale: bool) -> Self {
        assert!(min <= max, "min must be less than max");

        if log_scale {
            assert!(
                min > 0.0,
                "min must be greater than 0 for logarithmic scaling"
            );
        }

        if log_scale {
            min = libm::log10f(min);
            max = libm::log10f(max);
        }

        Self {
            adc,
            pin,
            min,
            max,
            log_scale,
        }
    }
}

impl<'d, T, P> dg_traits::FloatParameter for AdcFloatParameter<'d, T, P>
where
    T: Instance,
    P: AdcChannel<T> + 'd,
{
    async fn get(&mut self) -> f32 {
        let Self {
            adc,
            pin,
            min,
            max,
            log_scale,
        } = self;

        //  0                  2**16
        //  │                    │
        //  ┌───┬───┬───┬───┬───┐
        //  │ L │ … │ … │ … │ H │
        //  └───┴───┴───┴───┴───┘
        //  │                    │
        //  0                  H-L+1
        //

        let value = adc.blocking_read(pin) as f32;

        info!("ADC value: {}", value);

        // Correct for patch.Init pots, which are inverted and return 0-2**15
        //TODO: make that optional?
        let value = (32768.0 - value) * 2.0;

        let min = *min;
        let max = *max;
        let range = max - min;

        let mut res = min + range * value / 65536.0;

        info!("Mapped value before: {}", res);

        if *log_scale {
            res = libm::powf(10.0, res);
        }

        info!("Mapped value: {}", res);
        res
    }
}
