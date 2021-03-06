use std::cell::RefCell;
use std::cmp::min;
use std::path::Path;

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

    pub fn write(&self, address: u32, data: &[u8]) {
        let limit = min(data.len(), (self.size + self.start - address) as usize);
        self.bytes.borrow_mut()[(address - self.start) as usize..(limit + address as usize)].copy_from_slice(&data[..limit]);
    }

    pub fn read(&self, address: u32, data: &mut [u8]) {
        let limit = min(data.len(), (self.size + self.start - address) as usize);
        data[..limit].copy_from_slice(&self.bytes.borrow()[(address - self.start) as usize..(limit + address as usize)]);
    }

    pub fn load_file<P: AsRef<Path>>(&self, address: u32, filename: P) -> Result<(), std::io::Error> {
        let buffer = std::fs::read(filename)?;
        self.write(address, buffer.as_slice());
        Ok(())
    }
}

impl Peripheral for RAM {
    fn mem_read(&self, address: u32, _m1: bool) -> Option<u8> {
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
