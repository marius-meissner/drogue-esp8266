use crate::adapter::{Adapter, SocketError};
use embedded_hal::serial::Write;

use core::cell::{RefCell, BorrowMutError, RefMut};
use drogue_network::{Mode, SocketAddr, TcpStack};

/// NetworkStack for and ESP8266
pub struct NetworkStack<'a, Tx>
where
    Tx: Write<u8>,
{
    adapter: RefCell<Adapter<'a, Tx>>,
}

impl<'a, Tx> NetworkStack<'a, Tx>
where
    Tx: Write<u8>,
{
    pub(crate) fn new(adapter: Adapter<'a, Tx>) -> Self {
        Self {
            adapter: RefCell::new(adapter),
        }
    }
}

/// Handle to a socket.
#[derive(Debug)]
pub struct TcpSocket {
    link_id: usize,
    mode: Mode,
}

impl<'a, Tx> TcpStack for NetworkStack<'a, Tx>
where
    Tx: Write<u8>,
{
    type TcpSocket = TcpSocket;
    type Error = SocketError;

    fn open(&self, mode: Mode) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        let result = Ok(TcpSocket {
            link_id: adapter.open()?,
            mode,
        });
        result
    }

    fn connect(
        &self,
        socket: Self::TcpSocket,
        remote: SocketAddr,
    ) -> Result<Self::TcpSocket, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        adapter.connect_tcp(socket.link_id, remote)?;
        let result = Ok(socket);
        result
    }

    fn is_connected(&self, socket: &Self::TcpSocket) -> Result<bool, Self::Error> {
        let adapter = self.adapter.borrow();
        adapter.is_connected(socket.link_id)
    }

    fn write(&self, socket: &mut Self::TcpSocket, buffer: &[u8]) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        let result = Ok(adapter
            .write(socket.link_id, buffer)
            .map_err(nb::Error::from)?);

        result
    }

    fn read(
        &self,
        socket: &mut Self::TcpSocket,
        buffer: &mut [u8],
    ) -> nb::Result<usize, Self::Error> {
        let mut adapter = self.adapter.borrow_mut();

        let result = match socket.mode {
            Mode::Blocking => {
                nb::block!(adapter.read(socket.link_id, buffer)).map_err(nb::Error::from)
            }
            Mode::NonBlocking => adapter.read(socket.link_id, buffer),
            Mode::Timeout(_) => unimplemented!(),
        };
        result
    }

    fn close(&self, socket: Self::TcpSocket) -> Result<(), Self::Error> {
        let mut adapter = self.adapter.borrow_mut();
        adapter.close(socket.link_id)
    }
}
