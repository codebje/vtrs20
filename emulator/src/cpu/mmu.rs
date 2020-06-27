use std::cell::RefCell;

use crate::types::Peripheral;

pub struct MMU {
    dcntl: RefCell<u8>,
}

impl MMU {
    pub fn new() -> MMU {
        MMU {
            dcntl: RefCell::new(0b1111_0000),
        }
    }

    pub fn reset(&self) {}

    pub fn to_physical(&self, addr: u16) -> u32 {
        addr.into()
    }

    #[allow(dead_code)]
    pub fn get_waits(&self) -> u8 {
        *self.dcntl.borrow()
    }
}

impl Peripheral for MMU {
    fn io_read(&self, _address: u16) -> Option<u8> {
        None
    }

    fn io_write(&self, address: u16, data: u8) {
        if address == 0b0000_0000_0010_0000 {
            let mut dcntl = self.dcntl.borrow_mut();
            *dcntl = data;
        }
    }
}
