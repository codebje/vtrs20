/**
 * Known incompatibilities:
 *  1. The CTS#/PS bit in CNTLB reads as the last prescale value set, not CTS#
 *  2. Multiprocessor mode is not implemented
 *  3. External speed select is not implemented
 *  4. Flow control is not implemented
 *  5. BRG is not implemented
 *  6. Transmit Enable/Receive Enable are ignored
 *  7. Interrupts aren't implemented
 */
use std::cell::RefCell;
use std::io::{ErrorKind, Read, Write};

use crate::bus::Bus;
use crate::types::*;

#[derive(PartialEq)]
pub enum Channel {
    CH0,
    CH1,
}

pub struct ASCI {
    channel: Channel,
    serial: RefCell<mio_serial::Serial>,
    cntla: RefCell<u8>,
    cntlb: RefCell<u8>,
    stat: RefCell<u8>,
    tdr: RefCell<u8>,
    rdr: RefCell<u8>,
}

impl ASCI {
    pub fn new(ch: Channel, path: &str) -> ASCI {
        let settings = mio_serial::SerialPortSettings::default();
        let rx = mio_serial::Serial::from_path(path, &settings).unwrap();
        let cntla = if ch == Channel::CH0 { 0b0001_0000 } else { 0 };
        ASCI {
            channel: ch,
            serial: RefCell::new(rx),
            cntla: RefCell::new(cntla),
            cntlb: RefCell::new(0b0000_0111),
            stat: RefCell::new(0b0000_0010),
            tdr: RefCell::new(0),
            rdr: RefCell::new(0),
        }
    }

    fn setup(&self) {
        let cntlb = *self.cntlb.borrow();
        let _mode = (*self.cntla.borrow() & 0b0111) | (cntlb & 0b1_0000);
        //let bits = if mode & 0b100 == 0 { DataBits::Seven } else { DataBits::Eight };
        //let stop = if mode & 1 == 0 { StopBits::One } else { StopBits::Two };
        //let parity = match mode & 0b1_0010 {
        //0b0_0010 => Parity::Even,
        //0b1_0010 => Parity::Odd,
        //_ => Parity::None,
        //};
        //let mut master = self.master.borrow_mut();
        //master.set_data_bits(bits).expect("PTYs shouldn't fail to set data bits");
        //master.set_parity(parity).expect("PTYs shouldn't fail to set parity");
        //master.set_stop_bits(stop).expect("PTYs shouldn't fail to set stop bits");

        // baud rate setting on a pty fails for no useful reason
    }

    fn xmit(&self) {
        *self.stat.borrow_mut() &= 0b1111_1101; // Reset TDRE
                                                //let mut stat = self.stat.borrow_mut();
                                                //*stat |= 0b0000_0010; // ensure TDRE is always set
                                                //let byte = *self.tdr.borrow();
                                                //self.serial.borrow_mut().write(&[byte]).unwrap();
    }
    fn recv(&self) {
        *self.stat.borrow_mut() &= 0b0111_1111;
    }
}

/*
 * CNTLA0»·»·······equ»····$00»····»·······»·······; ASCI Control Register A Ch 0
 * CNTLA1»·»·······equ»····$01»····»·······»·······; ASCI Control Register A Ch 1
 * CNTLB0»·»·······equ»····$02»····»·······»·······; ASCI Control Register B Ch 0
 * CNTLB1»·»·······equ»····$03»····»·······»·······; ASCI Control Register B Ch 1
 * STAT0»··»·······equ»····$04»····»·······»·······; ASCI Status Register Ch 0
 * STAT1»··»·······equ»····$05»····»·······»·······; ASCI Status Register Ch 1
 * TDR0»···»·······equ»····$06»····»·······»·······; ASCI Transmit Data Register Ch 0
 * TDR1»···»·······equ»····$07»····»·······»·······; ASCI Transmit Data Register Ch 1
 * RDR0»···»·······equ»····$08»····»·······»·······; ASCI Receive Data Register Ch 0
 * RDR1»···»·······equ»····$09»····»·······»·······; ASCI Receive Data Register Ch 1
 * ASEXT0»·»·······equ»····$12»····»·······»·······; ASCI Extension Control Register Ch 0
 * ASEXT1»·»·······equ»····$13»····»·······»·······; ASCI Extension Control Register Ch 1
 * ASTC0L»·»·······equ»····$1A»····»·······»·······; ASCI Time Constant Low Ch 0
 * ASTC0H»·»·······equ»····$1B»····»·······»·······; ASCI Time Constant High Ch 0
 * ASTC1L»·»·······equ»····$1C»····»·······»·······; ASCI Time Constant Low Ch 1
 * ASTC1H»·»·······equ»····$1D»····»·······»·······; ASCI Time Constant High Ch 1
 */

impl Peripheral for ASCI {
    fn cycle(&self, _bus: &Bus) -> Option<Interrupt> {
        let mut stat = self.stat.borrow_mut();

        // if TDRE is reset, try to send a byte
        if *stat & 0b0000_0010 == 0 {
            let byte = *self.tdr.borrow();
            match self.serial.borrow_mut().write(&[byte]) {
                Ok(_) => {
                    *stat |= 0b0000_0010;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                Err(ref e) => {
                    println!("Serial write error {}", e);
                }
            }
        }

        // if RDRF isn't set, try to read a byte
        if *stat & 0b1000_0000 == 0 {
            let mut rx = self.serial.borrow_mut();
            let mut buf = [0u8; 1];
            match rx.read(&mut buf) {
                Ok(_) => {
                    *self.rdr.borrow_mut() = buf[0];
                    *stat |= 0b1000_0000;
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => (),
                Err(ref e) => {
                    println!("Serial read error {}", e);
                }
            }
        }

        None
    }

    #[rustfmt::skip]
    fn io_read(&self, address: u16) -> Option<u8> {
        match address {
            0x0000 => if self.channel == Channel::CH0 { Some(*self.cntla.borrow()) } else { None },
            0x0001 => if self.channel == Channel::CH1 { Some(*self.cntla.borrow()) } else { None },
            0x0002 => if self.channel == Channel::CH0 { Some(*self.cntlb.borrow()) } else { None },
            0x0003 => if self.channel == Channel::CH1 { Some(*self.cntlb.borrow()) } else { None },
            0x0004 => if self.channel == Channel::CH0 { Some(*self.stat.borrow())  } else { None },
            0x0005 => if self.channel == Channel::CH1 { Some(*self.stat.borrow())  } else { None },
            0x0006 => if self.channel == Channel::CH0 { Some(*self.tdr.borrow())   } else { None },
            0x0007 => if self.channel == Channel::CH1 { Some(*self.tdr.borrow())   } else { None },
            0x0008 => if self.channel == Channel::CH0 { self.recv(); Some(*self.rdr.borrow())   } else { None },
            0x0009 => if self.channel == Channel::CH1 { self.recv(); Some(*self.rdr.borrow())   } else { None },
            _ => None,
        }
    }

    #[rustfmt::skip]
    fn io_write(&self, address: u16, data: u8) {
        match address {
            0x0000 => if self.channel == Channel::CH0 { *self.cntla.borrow_mut() = data; self.setup(); },
            0x0001 => if self.channel == Channel::CH1 { *self.cntla.borrow_mut() = data; self.setup(); },
            0x0002 => if self.channel == Channel::CH0 { *self.cntlb.borrow_mut() = data; self.setup(); },
            0x0003 => if self.channel == Channel::CH1 { *self.cntlb.borrow_mut() = data; self.setup(); },
            0x0004 => if self.channel == Channel::CH0 { *self.stat.borrow_mut() = data },
            0x0005 => if self.channel == Channel::CH1 { *self.stat.borrow_mut() = data },
            0x0006 => if self.channel == Channel::CH0 { *self.tdr.borrow_mut() = data; self.xmit(); },
            0x0007 => if self.channel == Channel::CH1 { *self.tdr.borrow_mut() = data; self.xmit(); },
            0x0008 => if self.channel == Channel::CH0 { *self.rdr.borrow_mut() = data },
            0x0009 => if self.channel == Channel::CH1 { *self.rdr.borrow_mut() = data },
            _ => (),
        };
    }
}

#[cfg(test)]
mod test {
    use crate::asci::*;

    #[test]
    fn mk_asci() {
        let asci = ASCI::new(Channel::CH0, "/dev/ttys009");

        // set up 38400, 8n1, TE/RE
        asci.io_write(0x00, 0b0110_0110);
        asci.io_write(0x02, 0b0010_0000);
    }
}
