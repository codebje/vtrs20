#[derive(Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    I,
    R,
    IX,
    IY,
    SP,
    PC,
}

//#[derive(Debug, Eq, PartialEq)]
//#[allow(dead_code)]
bitflags! {
    pub struct Flags: u8 {
        const CF = 0b0000_0001;     // carry
        const NF = 0b0000_0010;     // negative
        const PF = 0b0000_0100;     // parity
        const VF = 0b0000_0100;     // overflow
        const HF = 0b0001_0000;     // half-carry
        const ZF = 0b0100_0000;     // zero
        const SF = 0b1000_0000;     // signed
    }

}
