use std::convert::TryFrom;
use std::rc::Rc;

use crate::bus::Bus;
use crate::types::Peripheral;

mod mmu;

#[derive(Debug, Eq, PartialEq)]
#[allow(dead_code)]
enum Addressing {
    Implied,
    Direct,
    Indirect,
    Indexed,
    Extended,
    Immediate,
    Relative,
    IO,
}

#[repr(u8)]
enum Condition {
    NonZero = 0,
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

#[derive(Debug)]
enum Mode {
    Reset,
    OpCodeFetch,
    //IntAckNMI,
    //IntAckINT0,
    //IntAckOther,
    //BusRelease,
    //Halt,
    //Sleep,
    //DMARead,
    //DMAWrite,
    //IORead,
    //IOWrite,
}

// General registers
#[derive(Debug)]
struct GR {
    a: u8,   // accumulator
    f: u8,   // flags
    bc: u16, // B, C
    de: u16, // D, E
    hl: u16, // H, L
}

// Special registers
#[derive(Debug)]
struct SR {
    i: u8,
    r: u8,
    ix: u16,
    iy: u16,
    sp: u16,
    pc: u16,
}

#[allow(dead_code)]
pub struct CPU {
    // internal state
    mode: Mode,
    mmu: Rc<mmu::MMU>,
    gr: GR,
    gr_: GR,
    sr: SR,
}

impl CPU {
    // Create a new CPU. The CPU will be held in reset initially.
    pub fn new(bus: &mut Bus) -> CPU {
        let mmu = Rc::new(mmu::MMU::new());
        bus.add(Rc::clone(&mmu) as Rc<dyn Peripheral>);
        CPU {
            mode: Mode::Reset,
            mmu: mmu,
            gr: GR {
                a: 0,
                f: 0,
                bc: 0,
                de: 0,
                hl: 0,
            },
            gr_: GR {
                a: 0,
                f: 0,
                bc: 0,
                de: 0,
                hl: 0,
            },
            sr: SR {
                i: 0,
                r: 0,
                ix: 0,
                iy: 0,
                sp: 0x0000,
                pc: 0x0000,
            },
        }
    }

    // reset the CPU.
    pub fn reset<'b>(&mut self) {
        self.mode = Mode::OpCodeFetch;
        self.sr.pc = 0x0000;
        self.sr.sp = 0x0000;

        // reset own peripherals
        self.mmu.reset();
    }

    // Run one machine cycle. This will assert various signals on the bus to do its job.
    pub fn cycle(&mut self, bus: &mut Bus) {
        println!("Executing {:?} machine cyle", self.mode);
        // check for: interrupt, DMA, ...?
        match self.mode {
            Mode::Reset => (),
            Mode::OpCodeFetch => self.fetch_opcode(bus),
            //Mode::OpCodeFetch(opcode) => self.decode(opcode),
        }
    }

    // enter an error state
    fn error(&mut self) {
        self.mode = Mode::Reset;
        println!("Illegal instruction. Halt.");
    }

    // Pull an opcode from the bus
    fn fetch_opcode(&mut self, bus: &mut Bus) {
        let opcode = bus.mem_read(self.mmu.to_physical(self.sr.pc));

        println!("Opcode: {}", opcode);

        // decode the instruction
        match opcode {
            x if x & 0b11_111_000 == 0b10_000_000 => self.add_a_g(bus, x & 0b00_000_111),
            x if x & 0b11_000_111 == 0b00_000_110 => self.ld_g_m(bus, (x & 0b00_111_000) >> 3),
            _ => self.error(),
        }

        self.mode = Mode::Reset;
    }

    // Execute ADD A,g or ADD A,(HL)
    fn add_a_g(&mut self, bus: &mut Bus, g: u8) {
        println!("g = {}", g);
    }

    fn ld_g_m(&mut self, bus: &mut Bus, g: u8) {
        let imm = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1));
        println!("g = {}, imm = {}", g, imm);
    }
}
