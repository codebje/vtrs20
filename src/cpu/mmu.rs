pub struct MMU {}

impl MMU {
    pub fn new() -> MMU {
        MMU {}
    }

    pub fn reset(&self) {}

    pub fn to_physical(&self, addr: u16) -> u32 {
        addr.into()
    }
}
