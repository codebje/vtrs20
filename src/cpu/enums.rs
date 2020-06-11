use std::convert::{From, TryFrom};
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
    AF,
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

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub(super) enum RegIndirect {
    BC,
    DE,
    HL,
}

impl From<RegIndirect> for Register {
    fn from(reg: RegIndirect) -> Register {
        match reg {
            RegIndirect::BC => Register::BC,
            RegIndirect::DE => Register::DE,
            RegIndirect::HL => Register::HL,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub(super) enum RegIndex {
    IX,
    IY,
}

impl From<RegIndex> for Register {
    fn from(reg: RegIndex) -> Register {
        match reg {
            RegIndex::IX => Register::IX,
            RegIndex::IY => Register::IY,
        }
    }
}

/// The source or destination operand
#[allow(dead_code)]
pub(super) enum Operand {
    Direct(Register),
    Indirect(RegIndirect),
    Indexed(RegIndex),
    Extended(),
    Extended16(),
    Immediate(),
    Immediate16(),
    Relative(),
}

pub(super) enum Condition {
    NonZero,
    Zero,
    NonCarry,
    Carry,
    ParityOdd,
    ParityEven,
    SignPlus,
    SignMinus,
}

impl TryFrom<u8> for Condition {
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0b000 => Ok(Condition::NonZero),
            0b001 => Ok(Condition::Zero),
            0b010 => Ok(Condition::NonCarry),
            0b011 => Ok(Condition::Carry),
            0b100 => Ok(Condition::ParityOdd),
            0b101 => Ok(Condition::ParityEven),
            0b110 => Ok(Condition::SignPlus),
            0b111 => Ok(Condition::SignMinus),
            _ => Err(()),
        }
    }
}
