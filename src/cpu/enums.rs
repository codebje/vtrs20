use std::convert::From;
use std::fmt;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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

// `ggg` bitfield register decode, 0b110 excluded
#[derive(Copy, Clone)]
pub(super) enum RegG {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    A = 0b111,
}

impl From<RegG> for Register {
    fn from(reg: RegG) -> Register {
        match reg {
            RegG::B => Register::B,
            RegG::C => Register::C,
            RegG::D => Register::D,
            RegG::E => Register::E,
            RegG::H => Register::H,
            RegG::L => Register::L,
            RegG::A => Register::A,
        }
    }
}

// `ggg` bitfield register decode, plus 0b110 = HL
#[derive(Copy, Clone, Debug)]
pub(super) enum RegGHL {
    B = 0b000,
    C = 0b001,
    D = 0b010,
    E = 0b011,
    H = 0b100,
    L = 0b101,
    HL = 0b110,
    A = 0b111,
}

impl From<RegG> for RegGHL {
    fn from(reg: RegG) -> RegGHL {
        match reg {
            RegG::B => RegGHL::B,
            RegG::C => RegGHL::C,
            RegG::D => RegGHL::D,
            RegG::E => RegGHL::E,
            RegG::H => RegGHL::H,
            RegG::L => RegGHL::L,
            RegG::A => RegGHL::A,
        }
    }
}

// `ww` bitfield register decode
#[derive(Copy, Clone)]
pub(super) enum RegW {
    BC = 0b00,
    DE = 0b01,
    HL = 0b10,
    SP = 0b11,
}

impl From<RegW> for Register {
    fn from(reg: RegW) -> Register {
        match reg {
            RegW::BC => Register::BC,
            RegW::DE => Register::DE,
            RegW::HL => Register::HL,
            RegW::SP => Register::SP,
        }
    }
}
