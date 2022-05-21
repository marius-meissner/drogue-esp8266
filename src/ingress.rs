use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::{buffer::Buffer, protocol::Response};
use heapless::{
    consts::{U16, U2},
    spsc::Producer,
};

use embedded_hal::serial::Read;
use nb::Error;

/// Gets incremented on call to digest()
pub(crate) static DIGEST_COUNTER: AtomicU32 = AtomicU32::new(0);

/// True if timeout was triggered
pub(crate) static TIMEOUT_TRIGGERED: AtomicBool = AtomicBool::new(false);

pub struct Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    rx: Rx,
    response_producer: Producer<'a, Response, U2>,
    notification_producer: Producer<'a, Response, U16>,
    buffer: Buffer,
}

impl<'a, Rx> Ingress<'a, Rx>
    where
        Rx: Read<u8>,
{
    pub fn new(
        rx: Rx,
        response_producer: Producer<'a, Response, U2>,
        notification_producer: Producer<'a, Response, U16>,
    ) -> Self {
        Self {
            rx,
            response_producer,
            notification_producer,
            buffer: Buffer::new(),
        }
    }

    /// Method to be called from USART or appropriate ISR.
    pub fn isr(&mut self) -> Result<(), u8> {
        loop {
            let result = self.rx.read();
            match result {
                Ok(d) => {
                    self.write(d)?;
                }
                Err(e) => {
                    match e {
                        Error::Other(_) => {
                        }
                        Error::WouldBlock => {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, octet: u8) -> Result<(), u8> {
        self.buffer.write(octet)?;
        Ok(())
    }

    /// Digest and process the existing ingressed buffer to
    /// emit appropriate responses and notifications back
    pub fn digest(&mut self) {
        self.increment_counter();
        let result = self.buffer.parse();


        if let Ok(response) = result {
            if ! matches!(response, Response::None ) {
                log::info!("--> {:?}", response);
            }
            match response {
                Response::None => {}
                Response::Ok
                | Response::Error
                | Response::FirmwareInfo(..)
                | Response::Connect(..)
                | Response::ReadyForData
                | Response::ReceivedDataToSend(..)
                | Response::DataReceived(..)
                | Response::SendOk
                | Response::SendFail
                | Response::WifiConnectionFailure(..)
                | Response::IpAddress(..)
                | Response::Resolvers(..)
                | Response::DnsFail
                | Response::UnlinkFail
                | Response::IpAddresses(..) => {
                    if let Err(response) = self.response_producer.enqueue(response) {
                        log::error!("failed to enqueue response {:?}", response);
                    }
                }
                Response::Closed(..) | Response::DataAvailable { .. } => {
                    if let Err(response) = self.notification_producer.enqueue(response) {
                        log::error!("failed to enqueue notification {:?}", response);
                    }
                }
                Response::WifiConnected => {
                    log::info!("wifi connected");
                }
                Response::WifiDisconnect => {
                    log::info!("wifi disconnect");
                }
                Response::GotIp => {
                    log::info!("wifi got ip");
                }
            }
        }
    }

    /// Increments the DIGEST_COUNTER
    fn increment_counter(&self) {
        let current = DIGEST_COUNTER.load(Ordering::Relaxed);

        if current == u32::MAX {
            DIGEST_COUNTER.store(0, Ordering::Relaxed);
            return;
        }

        DIGEST_COUNTER.store(current + 1, Ordering::Relaxed);
    }
}
