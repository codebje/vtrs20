use std::cell::RefCell;

use crate::types::Peripheral;

const CBR: u16 = 0x38;
const BBR: u16 = 0x39;
const CBAR: u16 = 0x3A;

pub struct MMU {
    cbr: RefCell<u8>,
    bbr: RefCell<u8>,
    cbar: RefCell<u8>,
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            cbr: RefCell::new(0b0000_0000),
            bbr: RefCell::new(0b0000_0000),
            cbar: RefCell::new(0b1111_0000),
        }
    }

    pub fn reset(&self) {
        *self.cbr.borrow_mut() = 0;
        *self.bbr.borrow_mut() = 0;
        *self.cbar.borrow_mut() = 0xf0;
    }

    pub fn to_physical(&self, addr: u16) -> u32 {
        let cbar = *self.cbar.borrow();
        let bank = ((cbar as u16) & 0x0f) << 12;
        let common = ((cbar as u16) & 0xf0) << 8;
        if addr < bank {
            addr.into()
        } else if addr < common {
            addr as u32 + ((*self.bbr.borrow() as u32) << 12) - bank as u32
        } else {
            addr as u32 + ((*self.cbr.borrow() as u32) << 12) - common as u32
        }
    }
}

impl Peripheral for MMU {
    fn io_read(&self, address: u16) -> Option<u8> {
        match address {
            CBR => Some(*self.cbr.borrow()),
            BBR => Some(*self.bbr.borrow()),
            CBAR => Some(*self.cbar.borrow()),
            _ => None,
        }
    }

    fn io_write(&self, address: u16, data: u8) {
        match address {
            CBR => *self.cbr.borrow_mut() = data,
            BBR => *self.bbr.borrow_mut() = data,
            CBAR => *self.cbar.borrow_mut() = data,
            _ => (),
        }
    }
}
