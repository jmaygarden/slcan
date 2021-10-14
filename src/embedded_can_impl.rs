use crate::{CanFrame, CanSocket};
use embedded_can::{Can, Frame, Id};
use serial_core::SerialPort;
use std::io;

impl Frame for CanFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Result<Self, ()> {
        Ok(CanFrame::new(id.into(), data.len(), data))
    }

    fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Result<Self, ()> {
        // currently unsupported
        Err(())
    }

    fn is_extended(&self) -> bool {
        matches!(self.id, Id::Extended(_))
    }

    fn is_remote_frame(&self) -> bool {
        // currently unsupported
        false
    }

    fn id(&self) -> Id {
        self.id
    }

    fn dlc(&self) -> usize {
        self.dlc
    }

    fn data(&self) -> &[u8] {
        &self.data
    }
}

impl<P: SerialPort> Can for CanSocket<P> {
    type Frame = CanFrame;

    type Error = io::Error;

    fn try_transmit(
        &mut self,
        frame: &Self::Frame,
    ) -> nb::Result<Option<Self::Frame>, Self::Error> {
        self.write(frame.id, frame.data())
            .map(|_| None)
            .map_err(|io_err| {
                if io_err.kind() == io::ErrorKind::WouldBlock {
                    nb::Error::WouldBlock
                } else {
                    nb::Error::Other(io_err)
                }
            })
    }

    fn try_receive(&mut self) -> nb::Result<Self::Frame, Self::Error> {
        self.read().map_err(|io_err| {
            if io_err.kind() == io::ErrorKind::WouldBlock {
                nb::Error::WouldBlock
            } else {
                nb::Error::Other(io_err)
            }
        })
    }
}
