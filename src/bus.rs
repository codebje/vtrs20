use std::rc::Rc;

use crate::types::Peripheral;

pub struct Bus {
    peripherals: Vec<Rc<dyn Peripheral>>,
}

#[allow(dead_code)]
impl Bus {
    pub fn new() -> Bus {
        Bus {
            peripherals: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        for peripheral in &self.peripherals {
            peripheral.reset();
        }
    }

    pub fn add(&mut self, peripheral: Rc<dyn Peripheral>) {
        self.peripherals.push(peripheral);
    }

    pub fn mem_read(&mut self, address: u32) -> u8 {
        for peripheral in &self.peripherals {
            match peripheral.mem_read(address) {
                Some(data) => {
                    println!("mem_read({}) = {}", address, data);
                    return data;
                }
                None => (),
            }
        }
        println!("mem_read({}) not found", address);
        255
    }

    pub fn mem_write(&mut self, address: u32, data: u8) {
        println!("mem_write({}) = {}", address, data);
        for peripheral in &self.peripherals {
            peripheral.mem_write(address, data);
        }
    }

    pub fn io_read(&mut self, address: u16) -> u8 {
        for peripheral in &self.peripherals {
            match peripheral.io_read(address) {
                Some(data) => return data,
                None => (),
            }
        }
        255
    }

    pub fn io_write(&mut self, address: u16, data: u8) {
        println!("io_write({}) = {}", address, data);
        for peripheral in &self.peripherals {
            peripheral.io_write(address, data);
        }
    }
}
