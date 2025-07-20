use embassy_stm32::{gpio::Output, mode::Async, spi::Spi};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::{Channel, DynamicSender},
};
use fhx::Fhx;

static FHX_CHANNEL: Channel<CriticalSectionRawMutex, FhxSetMessage, 5> = Channel::new();

pub struct FhxCv {
    sender: DynamicSender<'static, FhxSetMessage>,
    address: fhx::CvAddress,
    channel: fhx::CvChannel,
}

impl FhxCv {
    pub fn new(address: fhx::CvAddress, channel: fhx::CvChannel) -> Self {
        Self {
            sender: FHX_CHANNEL.dyn_sender(),
            address,
            channel,
        }
    }
    pub async fn set_value(&self, value: u16) {
        self.sender
            .send(FhxSetMessage::Cv {
                address: self.address,
                channel: self.channel,
                value,
            })
            .await;
    }

    //TODO: set polarity
}

pub struct FhxGate {
    sender: DynamicSender<'static, FhxSetMessage>,
    address: fhx::GtAddress,
    channel: fhx::GtChannel,
}

impl FhxGate {
    pub fn new(address: fhx::GtAddress, channel: fhx::GtChannel) -> Self {
        Self {
            sender: FHX_CHANNEL.dyn_sender(),
            address,
            channel,
        }
    }

    pub async fn set_high(&self) {
        self.sender
            .send(FhxSetMessage::Gate {
                address: self.address,
                channel: self.channel,
                value: true,
            })
            .await;
    }

    pub async fn set_low(&self) {
        self.sender
            .send(FhxSetMessage::Gate {
                address: self.address,
                channel: self.channel,
                value: false,
            })
            .await;
    }
}

//
// FHX (move to separate file?)
//

pub enum FhxSetMessage {
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
pub async fn fhx_worker(
    mut fhx: Fhx<
        'static,
        Spi<'static, Async>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
        Output<'static>,
    >,
) {
    let receiver = FHX_CHANNEL.receiver();
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
