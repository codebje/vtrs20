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

// `ww` bitfield register decode
#[derive(Copy, Clone, Debug)]
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
#[derive(Debug, Copy, Clone)]
pub(super) enum Operand {
    Absolute(u8),
    Memory(u16),
    Direct(Register),
    Indirect(RegIndirect),
    Indexed(RegIndex),
    Extended(),
    Extended16(),
    Immediate(),
    Immediate16(),
    Relative(),
    Discard(),
}

#[derive(Debug)]
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

#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub(super) enum Exchange {
    AF_AFs,
    DE_HL,
    X,
    SP_HL,
    SP_IX,
    SP_IY,
}

/** Bit shifter operation.
 *
 * This selects between rotate and shift operations, including selecting carry or no-carry rotates, and logical or
 * arithmetic shifts.
 */
#[derive(Debug, PartialEq)]
pub(super) enum ShiftOp {
    Rot,
    RotC,
    ShiftA,
    ShiftL,
}

/**
 * Rotate/shift mode.
 *
 * There are '8080' style rotates in the base instruction set that operate only on the A register. These do not affect
 * the flags in the same was as the full Z80 rotation set in the 'bits' extended operand range. This enum allows a
 * rotate function to distinguish the two modes.
 */
#[derive(Debug)]
pub(super) enum ShiftMode {
    R8080,
    RZ80,
}
