use std::rc::Rc;

pub use crate::types::*;

pub struct Bus {
    peripherals: Vec<Rc<dyn Peripheral>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus { peripherals: Vec::new() }
    }

    pub fn reset(&self) {
        for peripheral in &self.peripherals {
            peripheral.reset();
        }
    }

    pub fn cycle(&self) -> Option<Interrupt> {
        for peripheral in &self.peripherals {
            peripheral.cycle(self);
        }
        None
    }

    pub fn add(&mut self, peripheral: Rc<dyn Peripheral>) {
        self.peripherals.push(peripheral);
    }

    pub fn mem_read(&self, address: u32) -> u8 {
        for peripheral in &self.peripherals {
            match peripheral.mem_read(address) {
                Some(data) => {
                    return data;
                }
                None => (),
            }
        }
        255
    }

    pub fn mem_write(&self, address: u32, data: u8) {
        for peripheral in &self.peripherals {
            peripheral.mem_write(address, data);
        }
    }

    pub fn io_read(&self, address: u16) -> u8 {
        for peripheral in &self.peripherals {
            match peripheral.io_read(address) {
                Some(data) => return data,
                None => (),
            }
        }
        255
    }

    pub fn io_write(&self, address: u16, data: u8) {
        for peripheral in &self.peripherals {
            peripheral.io_write(address, data);
        }
    }
}
