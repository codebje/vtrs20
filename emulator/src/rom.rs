use std::cell::RefCell;

use crate::types::Peripheral;

pub struct ROM {
    start: u32,
    size: u32,
    bytes: RefCell<Vec<u8>>,
    is_masking: RefCell<bool>,
}

impl ROM {
    pub fn new(base: u32, contents: Vec<u8>) -> ROM {
        ROM {
            start: base,
            size: contents.len() as u32,
            bytes: RefCell::new(contents),
            is_masking: RefCell::new(true),
        }
    }
}

impl Peripheral for ROM {
    fn reset(&self) {
        *self.is_masking.borrow_mut() = true;
    }

    fn mem_read(&self, address: u32) -> Option<u8> {
        let mut addr = address;

        // After reset the ROM forces A19 high until the processor
        // has driven it high on its own.
        if (addr & 0b1000_0000_0000_0000_0000) != 0 {
            *self.is_masking.borrow_mut() = false;
        }

        if *self.is_masking.borrow() {
            addr |= 0b1000_0000_0000_0000_0000;
        }

        if addr >= self.start && addr <= self.start + self.size {
            return Some(self.bytes.borrow()[(addr - self.start) as usize]);
        }
        None
    }

    fn mem_write(&self, _address: u32, _data: u8) {
        // TODO SST39xF0x0 Flash command sequences for write control
    }
}
