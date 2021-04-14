use crate::{CanFrame, CanSocket};
use embedded_can::{Can, ExtendedId, Frame, Id, StandardId};
use serial_core::SerialPort;
use std::io;

const EFF_FLAG: u32 = 0x80000000;

impl Frame for CanFrame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Result<Self, ()> {
        let id = match id.into() {
            Id::Standard(id) => id.as_raw().into(),
            Id::Extended(id) => id.as_raw() | EFF_FLAG,
        };
        Ok(CanFrame::new(id, data.len(), data))
    }

    fn new_remote(_id: impl Into<Id>, _dlc: usize) -> Result<Self, ()> {
        // currently unsupported
        Err(())
    }

    fn is_extended(&self) -> bool {
        self.id & EFF_FLAG != 0
    }

    fn is_remote_frame(&self) -> bool {
        // currently unsupported
        false
    }

    fn id(&self) -> Id {
        if self.id & EFF_FLAG != 0 {
            Id::Extended(unsafe { ExtendedId::new_unchecked(self.id) })
        } else {
            Id::Standard(unsafe { StandardId::new_unchecked(self.id as u16) })
        }
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
