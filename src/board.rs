use crate::bus::Bus;
use crate::cpu::CPU;
use crate::types::Signal;
use crate::types::Tristate;

pub trait Peripheral {
    fn reset(&mut self, bus: &mut Bus);
    fn read(&mut self, address: u32) -> Option<u8>;
    fn write(&mut self, address: u32, data: u8);
    fn edge(&mut self, signal: Signal, state: Tristate);
}

pub struct Board<'a> {
    peripherals: Vec<&'a mut Box<dyn Peripheral>>,
    cpu: &'a mut CPU,
    bus: &'a mut Bus,
}

impl<'a> Board<'a> {
    pub fn reset(&mut self) {
        self.cpu.reset(&mut self.bus);
        for peripheral in &mut self.peripherals {
            peripheral.reset(&mut self.bus);
        }
    }
}
