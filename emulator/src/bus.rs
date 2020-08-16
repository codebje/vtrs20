use std::cell::RefCell;
use std::rc::Rc;

use enumset::EnumSet;

pub use crate::types::*;

pub struct Bus {
    peripherals: Vec<Rc<dyn Peripheral>>,
    ints: RefCell<EnumSet<Interrupt>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            peripherals: Vec::new(),
            ints: RefCell::new(EnumSet::new()),
        }
    }

    pub fn reset(&self) {
        for peripheral in &self.peripherals {
            peripheral.reset();
        }
    }

    pub fn cycle(&self) -> Option<Interrupt> {
        let mut ints = self.ints.borrow_mut();
        for peripheral in &self.peripherals {
            match peripheral.cycle(self) {
                Some(int) => {
                    *ints |= int;
                }
                None => (),
            }
        }

        // service the next highest priority pending interrupt
        ints.iter().next()
    }

    pub fn intack(&self, int: Interrupt) {
        *self.ints.borrow_mut() -= int;
    }

    pub fn add(&mut self, peripheral: Rc<dyn Peripheral>) {
        self.peripherals.push(peripheral);
    }

    pub fn mem_read(&self, address: u32, m1: bool) -> u8 {
        for peripheral in &self.peripherals {
            match peripheral.mem_read(address, m1) {
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
