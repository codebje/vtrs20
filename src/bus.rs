use std::collections::HashMap;

use crate::types::Signal;
use crate::types::Tristate;

pub struct Signals {
    map: HashMap<Signal, Tristate>,
}

impl Signals {
    pub fn new() -> Signals {
        Signals {
            map: HashMap::new(),
        }
    }

    pub fn raise(&mut self, signal: Signal) {
        self.map.insert(signal, Tristate::High);
    }

    pub fn lower(&mut self, signal: Signal) {
        self.map.insert(signal, Tristate::Low);
    }

    pub fn open(&mut self, signal: Signal) {
        self.map.remove(&signal);
    }

    pub fn sample(&self, signal: Signal) -> &Tristate {
        self.map.get(&signal).unwrap_or(&Tristate::HiZ)
    }
}

pub struct Bus {
    pub signals: Signals,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            signals: Signals::new(),
        }
    }

    pub fn read(&self, address: u32) -> u8 {
        //for peripheral in &self.peripherals {
        //match peripheral.read(address) {
        //Some(data) => return data,
        //None => (),
        //}
        //}
        255
    }

    pub fn write(&mut self, address: u32, data: u8) {
        //for peripheral in &mut self.peripherals {
        //peripheral.write(address, data);
        //}
    }
}
