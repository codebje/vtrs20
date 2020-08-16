use crate::bus::Bus;

use enumset::EnumSetType;

#[derive(PartialOrd, Ord, EnumSetType)]
pub enum Interrupt {
    TRAP,
    NMI,
    INT0,
    INT1,
    INT2,
    PTR0,
    PTR1,
    DMA0,
    DMA1,
    CSIO,
    ASCI0,
    ASCI1,
}

pub trait Peripheral {
    fn reset(&self) {}
    fn cycle(&self, _bus: &Bus) -> Option<Interrupt> {
        None
    }
    fn mem_read(&self, _address: u32, _m1: bool) -> Option<u8> {
        None
    }
    fn mem_write(&self, _address: u32, _data: u8) {}
    fn io_read(&self, _address: u16) -> Option<u8> {
        None
    }
    fn io_write(&self, _address: u16, _data: u8) {}
}
