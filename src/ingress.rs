use heapless::Vec;
use crate::{
    buffer::Buffer,
    protocol::Response,
};
use heapless::{
    consts::{
        U2,
        U16,
    },
    spsc::Producer,
};
use embedded_hal::serial::Read;

use log::info;


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
    pub fn new(rx: Rx,
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
    pub fn isr(&mut self) {
        if let Ok(d) = self.rx.read() {
            self.write(d);
            //info!( "{}", d as char);
        }
    }

    fn write(&mut self, octet: u8) -> Result<(), u8> {
        self.buffer.write(octet)?;
        Ok(())
    }

    /// Digest and process the existing ingressed buffer to
    /// emit appropriate responses and notifications back
    pub fn digest(&mut self) {
        let result = self.buffer.parse();

        match result {
            Ok(response) => {
                match response {
                    Response::None => {}
                    Response::Ok |
                    Response::Error |
                    Response::FirmwareInfo(..) |
                    Response::Connect(..) |
                    Response::ReadyForData  |
                    Response::DataReceived(..) |
                    Response::SendOk(..) |
                    Response::WifiConnectionFailure(..) |
                    Response::IpAddresses(..) => {
                        self.response_producer.enqueue(response);
                    }
                    Response::Closed(..) |
                    Response::DataAvailable { .. } => {
                        self.notification_producer.enqueue(response);
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

            Err(e) => {}
        }
    }
}