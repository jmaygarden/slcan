extern crate serial_core as serial;

use serial::prelude::*;
use std::io;

// maximum rx buffer len: extended CAN frame with timestamp
const SLCAN_MTU: usize = "T1111222281122334455667788EA5F\r".len() + 1;
const SLCAN_CMD_LEN: usize = 1;
const SLCAN_SDD_ID_LEN: usize = 3;

const BELL: u8 = 0x07;
const CARRIAGE_RETURN: u8 = '\r' as u8;
const TRANSMIT_COMMAND: u8 = 't' as u8;

pub struct CanFrame {
    pub id: u32,
    pub dlc: usize,
    pub data: [u8; 8],
}

pub struct CanSocket<P: SerialPort> {
    port: P,
    rbuff: [u8; SLCAN_MTU],
    rcount: usize,
    error: bool,
}

fn hextou8(s: u8) -> Result<u8, ()> {
    let c = s as char;

    if c >= '0' && c <= '9' {
        Ok(s - '0' as u8)
    } else if c >= 'a' && c <= 'f' {
        Ok(s - 'a' as u8 + 10)
    } else if c >= 'A' && c <= 'F' {
        Ok(s - 'A' as u8 + 10)
    } else {
        Err(())
    }
}

fn hex2tou8(s: &[u8]) -> Result<u8, ()> {
    let msn = hextou8(s[0])?;
    let lsn = hextou8(s[1])?;

    Ok((msn << 4) | lsn)
}

fn unpack_data(s: &[u8], len: usize) -> Result<[u8; 8], ()> {
    let mut buf = [u8::default(); 8];

    for i in 0..len {
        let offset = 2 * i;

        buf[i] = hex2tou8(&s[offset..])?;
    }

    Ok(buf)
}

fn hextou32(buf: &[u8]) -> Result<u32, ()> {
    let mut value = 0u32;

    for s in buf.iter() {
        value <<= 8;

        match hextou8(*s) {
            Ok(byte) => value |= byte as u32,
            Err(_) => return Err(()),
        }
    }

    Ok(value)
}

impl CanFrame {
    pub fn new(id: u32, dlc: usize, data: &[u8]) -> Self {
        let mut copy = [u8::default(); 8];
        copy[..data.len()].copy_from_slice(data);

        Self {
            id,
            dlc,
            data: copy,
        }
    }
}

impl std::fmt::Display for CanFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CanFrame{{ id: {}, dlc: {}, data: {:?} }}",
            self.id, self.dlc, self.data
        )
    }
}

impl<P: SerialPort> CanSocket<P> {
    pub fn open(port: P) -> Self {
        CanSocket {
            port,
            rbuff: [0; SLCAN_MTU],
            rcount: 0,
            error: false,
        }
    }

    pub fn read(&mut self) -> io::Result<CanFrame> {
        let mut buf = [0u8; 1];
        let mut len = self.port.read(&mut buf)?;

        while len == 1usize {
            let s = buf[0];

            if s == CARRIAGE_RETURN || s == BELL {
                if !self.error && self.rcount > 4 {
                    return self.bump();
                }

                self.error = false;
                self.rcount = 0;
            } else if !self.error {
                if self.rcount < SLCAN_MTU {
                    self.rbuff[self.rcount] = s;
                    self.rcount += 1;
                } else {
                    self.error = true;
                }
            }

            len = self.port.read(&mut buf)?;
        }

        Err(io::Error::new(io::ErrorKind::WouldBlock, ""))
    }

    fn bump(&mut self) -> io::Result<CanFrame> {
        let cmd = self.rbuff[0];

        match cmd {
            TRANSMIT_COMMAND => {
                let id =
                    match hextou32(&self.rbuff[SLCAN_CMD_LEN..SLCAN_CMD_LEN + SLCAN_SDD_ID_LEN]) {
                        Ok(value) => value,
                        Err(()) => return Err(io::Error::new(io::ErrorKind::WouldBlock, "")),
                    };
                let dlc = (self.rbuff[SLCAN_CMD_LEN + SLCAN_SDD_ID_LEN] - 0x30) as usize;

                if let Ok(data) =
                    unpack_data(&self.rbuff[SLCAN_CMD_LEN + SLCAN_SDD_ID_LEN + 1..], dlc)
                {
                    Ok(CanFrame { id, dlc, data })
                } else {
                    Err(io::Error::new(io::ErrorKind::InvalidData, ""))
                }
            }
            _ => Err(io::Error::new(io::ErrorKind::WouldBlock, "")),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
