use std::cell::RefCell;

use crate::types::*;

pub struct Reg {
    val: RefCell<u8>,
    addr: u16,
}

impl Reg {
    pub fn new(val: u8, addr: u16) -> Reg {
        Reg {
            val: RefCell::new(val),
            addr: addr,
        }
    }

    pub fn val(&self) -> u8 {
        *self.val.borrow()
    }
}

impl Peripheral for Reg {
    fn io_read(&self, address: u16) -> Option<u8> {
        if address == self.addr {
            Some(*self.val.borrow())
        } else {
            None
        }
    }

    fn io_write(&self, address: u16, data: u8) {
        if address == self.addr {
            *self.val.borrow_mut() = data;
        }
    }
}
