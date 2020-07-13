use crate::bus::Bus;

pub struct Interrupt {}

pub trait Peripheral {
    fn reset(&self) {}
    fn cycle(&self, _bus: &Bus) -> Option<Interrupt> {
        None
    }
    fn mem_read(&self, _address: u32) -> Option<u8> {
        None
    }
    fn mem_write(&self, _address: u32, _data: u8) {}
    fn io_read(&self, _address: u16) -> Option<u8> {
        None
    }
    fn io_write(&self, _address: u16, _data: u8) {}
}
