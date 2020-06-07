use std::cell::RefCell;

use crate::types::Peripheral;

pub struct RAM {
    start: u32,
    size: u32,
    bytes: RefCell<Vec<u8>>,
}

impl RAM {
    pub fn new(start: u32, size: u32) -> RAM {
        RAM {
            start: start,
            size: size,
            bytes: RefCell::new(vec![0u8; size as usize]),
        }
    }
}

impl Peripheral for RAM {
    fn mem_read(&self, address: u32) -> Option<u8> {
        if address >= self.start && address <= self.start + self.size {
            return Some(self.bytes.borrow()[(address - self.start) as usize]);
        }
        None
    }
    fn mem_write(&self, address: u32, data: u8) {
        if address >= self.start && address <= self.start + self.size {
            self.bytes.borrow_mut()[(address - self.start) as usize] = data;
        }
    }
}
