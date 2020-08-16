use std::cell::RefCell;
use std::num::Wrapping;

use crate::bus::Bus;
use crate::types::*;

/**
 * Limitations:
 *  1. Only does memory-to-memory
 *  2. Only implements DMA0
 *  3. Does not do interrupts
 *  4. Memory src/dst cannot wrap either direction
 */

pub struct DMA {
    sar0l: RefCell<u8>,
    sar0h: RefCell<u8>,
    sar0b: RefCell<u8>,
    dar0l: RefCell<u8>,
    dar0h: RefCell<u8>,
    dar0b: RefCell<u8>,
    bcr0l: RefCell<u8>,
    bcr0h: RefCell<u8>,
    mar1l: RefCell<u8>,
    mar1h: RefCell<u8>,
    mar1b: RefCell<u8>,
    iar1l: RefCell<u8>,
    iar1h: RefCell<u8>,
    iar1b: RefCell<u8>,
    bcr1l: RefCell<u8>,
    bcr1h: RefCell<u8>,
    dstat: RefCell<u8>,
    dmode: RefCell<u8>,
    dcntl: RefCell<u8>,
}

impl DMA {
    pub fn new() -> DMA {
        DMA {
            sar0l: RefCell::new(0),
            sar0h: RefCell::new(0),
            sar0b: RefCell::new(0),
            dar0l: RefCell::new(0),
            dar0h: RefCell::new(0),
            dar0b: RefCell::new(0),
            bcr0l: RefCell::new(0),
            bcr0h: RefCell::new(0),
            mar1l: RefCell::new(0),
            mar1h: RefCell::new(0),
            mar1b: RefCell::new(0),
            iar1l: RefCell::new(0),
            iar1h: RefCell::new(0),
            iar1b: RefCell::new(0),
            bcr1l: RefCell::new(0),
            bcr1h: RefCell::new(0),
            dstat: RefCell::new(0b0011_0000),
            dmode: RefCell::new(0),
            dcntl: RefCell::new(0),
        }
    }

    fn set_dstat(&self, byte: u8) {
        let mut dstat = self.dstat.borrow_mut();

        // DE1 can be set if DWE1 is reset, also sets DME
        if (byte & 0b1010_0000) == 0b1000_0000 {
            *dstat |= 0b1000_0001;
        }

        // DE0 can be set if DWE0 is reset, also sets DME
        if (byte & 0b0101_0000) == 0b0100_0000 {
            *dstat |= 0b0100_0001;
        }

        // Record DIE1, DIE0 settings
        *dstat |= byte & 0b0000_1100;
    }

    fn sar0(&self) -> u32 {
        (*self.sar0b.borrow() as u32) << 16 | (*self.sar0h.borrow() as u32) << 8 | (*self.sar0l.borrow() as u32)
    }

    fn set_sar0(&self, sar0: u32) {
        *self.sar0l.borrow_mut() = (sar0 & 0xff) as u8;
        *self.sar0h.borrow_mut() = (sar0 >> 8) as u8;
        *self.sar0b.borrow_mut() = (sar0 >> 16) as u8;
    }

    fn dar0(&self) -> u32 {
        (*self.dar0b.borrow() as u32) << 16 | (*self.dar0h.borrow() as u32) << 8 | (*self.dar0l.borrow() as u32)
    }

    fn set_dar0(&self, dar0: u32) {
        *self.dar0l.borrow_mut() = (dar0 & 0xff) as u8;
        *self.dar0h.borrow_mut() = (dar0 >> 8) as u8;
        *self.dar0b.borrow_mut() = (dar0 >> 16) as u8;
    }

    fn bcr0(&self) -> u16 {
        (*self.bcr0h.borrow() as u16) << 8 | (*self.bcr0l.borrow() as u16)
    }

    fn set_bcr0(&self, bcr0: u16) {
        *self.bcr0h.borrow_mut() = (bcr0 >> 8) as u8;
        *self.bcr0l.borrow_mut() = (bcr0 & 0xff) as u8;
    }
}

impl Peripheral for DMA {
    fn cycle(&self, bus: &Bus) -> Option<Interrupt> {
        // check DMA0 for an operation to execute
        let mut dstat = self.dstat.borrow_mut();
        let dmode = *self.dmode.borrow();

        // Here I assume that programming the DMAC with a count of zero will in fact transfer
        // 65,536 bytes, not zero.
        if *dstat & 0b0100_0001 == 0b0100_0001 {
            let mut src = self.sar0();
            let mut dst = self.dar0();
            let mut count = self.bcr0();
            let cmax = if count == 0 { 0x1_0000 } else { count as u64 };
            println!("**** DMAC transferring {:4x} bytes from {:5x} to {:5x}", cmax, src, dst);
            loop {
                let byte = bus.mem_read(src, false);
                bus.mem_write(dst, byte);

                match dmode & 0b0011_0000 {
                    0b0000_0000 => dst = dst + 1,
                    0b0001_0000 => dst = dst - 1,
                    _ => (),
                }

                match dmode & 0b0000_1100 {
                    0b0000_0000 => src = src + 1,
                    0b0001_0000 => src = src - 1,
                    _ => (),
                }

                count = (Wrapping(count) - Wrapping(1)).0;

                // Terminate the loop and transfer if count has reached zero
                if count == 0 {
                    // TODO trigger an interrupt
                    *dstat &= 0b1011_1111;
                    break;
                }

                // Terminate the loop if MMOD specifies cycle-stealing, not burst mode
                if dmode & 0b0000_0010 == 0 {
                    break;
                }
            }
            self.set_sar0(src);
            self.set_dar0(dst);
            self.set_bcr0(count);
        }

        None
    }

    fn io_read(&self, address: u16) -> Option<u8> {
        match address {
            0x20 => Some(*self.sar0l.borrow()),
            0x21 => Some(*self.sar0h.borrow()),
            0x22 => Some(*self.sar0b.borrow()),
            0x23 => Some(*self.dar0l.borrow()),
            0x24 => Some(*self.dar0h.borrow()),
            0x25 => Some(*self.dar0b.borrow()),
            0x26 => Some(*self.bcr0l.borrow()),
            0x27 => Some(*self.bcr0h.borrow()),
            0x28 => Some(*self.mar1l.borrow()),
            0x29 => Some(*self.mar1h.borrow()),
            0x2A => Some(*self.mar1b.borrow()),
            0x2B => Some(*self.iar1l.borrow()),
            0x2C => Some(*self.iar1h.borrow()),
            0x2D => Some(*self.iar1b.borrow()),
            0x2E => Some(*self.bcr1l.borrow()),
            0x2F => Some(*self.bcr1h.borrow()),
            0x30 => Some(*self.dstat.borrow()),
            0x31 => Some(*self.dmode.borrow()),
            0x32 => Some(*self.dcntl.borrow()),
            _ => None,
        }
    }

    fn io_write(&self, address: u16, data: u8) {
        match address {
            0x20 => *self.sar0l.borrow_mut() = data,
            0x21 => *self.sar0h.borrow_mut() = data,
            0x22 => *self.sar0b.borrow_mut() = data,
            0x23 => *self.dar0l.borrow_mut() = data,
            0x24 => *self.dar0h.borrow_mut() = data,
            0x25 => *self.dar0b.borrow_mut() = data,
            0x26 => *self.bcr0l.borrow_mut() = data,
            0x27 => *self.bcr0h.borrow_mut() = data,
            0x28 => *self.mar1l.borrow_mut() = data,
            0x29 => *self.mar1h.borrow_mut() = data,
            0x2A => *self.mar1b.borrow_mut() = data,
            0x2B => *self.iar1l.borrow_mut() = data,
            0x2C => *self.iar1h.borrow_mut() = data,
            0x2D => *self.iar1b.borrow_mut() = data,
            0x2E => *self.bcr1l.borrow_mut() = data,
            0x2F => *self.bcr1h.borrow_mut() = data,
            0x30 => self.set_dstat(data),
            0x31 => *self.dmode.borrow_mut() = data,
            0x32 => *self.dcntl.borrow_mut() = data,
            _ => (),
        }
    }
}
