#[allow(unused_variables)]
pub trait Peripheral {
    fn reset(&self) {}
    fn mem_read(&self, address: u32) -> Option<u8> {
        None
    }
    fn mem_write(&self, address: u32, data: u8) {}
    fn io_read(&self, address: u16) -> Option<u8> {
        None
    }
    fn io_write(&self, address: u16, data: u8) {}
}
