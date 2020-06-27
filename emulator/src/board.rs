use std::rc::Rc;

use crate::bus::Bus;
use crate::cpu::CPU;
use crate::types::Peripheral;

pub struct Board<'a> {
    cpu: &'a mut CPU,
    bus: &'a mut Bus,
}

impl<'a> Board<'a> {
    pub fn new(cpu: &'a mut CPU, bus: &'a mut Bus) -> Board<'a> {
        Board { cpu: cpu, bus: bus }
    }

    pub fn add(&mut self, peripheral: Rc<dyn Peripheral>) {
        self.bus.add(peripheral);
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.bus.reset();
    }

    // Run a clock cycle. The CPU
    pub fn cycle(&mut self) {
        self.cpu.cycle(&mut self.bus);
    }
}
