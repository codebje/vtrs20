use std::rc::Rc;

use crate::bus::Bus;
use crate::types::Peripheral;

// enums
pub mod enums;

// instruction set
mod alu;
mod alu16;
mod ctrl;
mod iops;
mod ld_16bit;
mod ld_8bit;
mod special;

// instruction decode and dispatch
mod dispatch;

// peripherals
mod mmu;

use enums::*;

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

    // Return the CPU flags
    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.gr.f)
    }

    // Return a register value. Always returns a u16, as Rust doesn't have dependent types.
    pub fn reg<R: Into<Register>>(&self, reg: R) -> u16 {
        match reg.into() {
            Register::A => self.gr.a as u16,
            Register::F => self.gr.f as u16,
            Register::B => self.gr.bc >> 8,
            Register::C => self.gr.bc & 0xff,
            Register::D => self.gr.de >> 8,
            Register::E => self.gr.de & 0xff,
            Register::H => self.gr.hl >> 8,
            Register::L => self.gr.hl & 0xff,
            Register::BC => self.gr.bc,
            Register::DE => self.gr.de,
            Register::HL => self.gr.hl,
            Register::I => self.sr.i as u16,
            Register::R => self.sr.r as u16,
            Register::IX => self.sr.ix,
            Register::IY => self.sr.iy,
            Register::SP => self.sr.sp,
            Register::PC => self.sr.pc,
        }
    }

    // Set a register value. Always returns a u16, as Rust doesn't have dependent types.
    pub fn write_reg<R: Into<Register>>(&mut self, reg: R, v: u16) {
        match reg.into() {
            Register::A => self.gr.a = v as u8,
            Register::F => self.gr.f = v as u8,
            Register::B => self.gr.bc = (self.gr.bc & 0xff) | v << 8,
            Register::C => self.gr.bc = (self.gr.bc & 0xff00) | (v & 0xff),
            Register::D => self.gr.de = (self.gr.de & 0xff) | v << 8,
            Register::E => self.gr.de = (self.gr.de & 0xff00) | (v & 0xff),
            Register::H => self.gr.hl = (self.gr.hl & 0xff) | v << 8,
            Register::L => self.gr.hl = (self.gr.hl & 0xff00) | (v & 0xff),
            Register::BC => self.gr.bc = v,
            Register::DE => self.gr.de = v,
            Register::HL => self.gr.hl = v,
            Register::I => self.sr.i = v as u8,
            Register::R => self.sr.r = v as u8,
            Register::IX => self.sr.ix = v,
            Register::IY => self.sr.iy = v,
            Register::SP => self.sr.sp = v,
            Register::PC => self.sr.pc = v,
        }
    }

    // Run one machine cycle. This will assert various signals on the bus to do its job.
    pub fn cycle(&mut self, bus: &mut Bus) {
        // check for: interrupt, DMA, ...?
        match self.mode {
            Mode::Reset => (),
            Mode::OpCodeFetch => self.dispatch(bus),
            //Mode::OpCodeFetch(opcode) => self.decode(opcode),
        }
    }

    // enter an error state
    fn error(&mut self) {
        self.mode = Mode::Reset;
        println!("Illegal instruction. Halt.");
    }

    // Load an operand using an addressing mode. Will adjust PC as needed.
    fn load_operand(&mut self, bus: &mut Bus, address: Addressing) -> u16 {
        match address {
            Addressing::Direct(reg) => self.reg(reg),
            Addressing::Indirect(reg) => bus.mem_read(self.mmu.to_physical(self.reg(reg))) as u16,
            Addressing::Indexed(reg) => {
                let d = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as i8;
                let addr = self.reg(reg) as i32 + d as i32;
                self.sr.pc += 1;
                bus.mem_read(self.mmu.to_physical(addr as u16)) as u16
            }
            Addressing::Extended() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                bus.mem_read(self.mmu.to_physical(addr)) as u16
            }
            Addressing::Immediate() => {
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                self.sr.pc += 1;
                m
            }
            Addressing::Immediate16() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                addr
            }
            Addressing::Relative() => {
                let d = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as i8;
                self.sr.pc += 1;
                (self.sr.pc as i32 + d as i32) as u16
            }
        }
    }

    fn load_ghl<U: Into<RegGHL>>(&mut self, bus: &mut Bus, g: U) -> u8 {
        match g.into() {
            RegGHL::B => ((self.gr.bc & 0xff00) >> 8) as u8,
            RegGHL::C => (self.gr.bc & 0x00ff) as u8,
            RegGHL::D => ((self.gr.de & 0xff00) >> 8) as u8,
            RegGHL::E => (self.gr.de & 0x00ff) as u8,
            RegGHL::H => ((self.gr.hl & 0xff00) >> 8) as u8,
            RegGHL::L => (self.gr.hl & 0x00ff) as u8,
            RegGHL::HL => bus.mem_read(self.mmu.to_physical(self.gr.hl)),
            RegGHL::A => self.gr.a,
        }
    }

    fn store_ghl<U: Into<RegGHL>>(&mut self, bus: &mut Bus, g: U, v: u8) {
        match g.into() {
            RegGHL::B => self.gr.bc = (self.gr.bc & 0x00ff) | ((v as u16) << 8),
            RegGHL::C => self.gr.bc = (self.gr.bc & 0xff00) | (v as u16),
            RegGHL::D => self.gr.de = (self.gr.de & 0x00ff) | ((v as u16) << 8),
            RegGHL::E => self.gr.de = (self.gr.de & 0xff00) | (v as u16),
            RegGHL::H => self.gr.hl = (self.gr.hl & 0x00ff) | ((v as u16) << 8),
            RegGHL::L => self.gr.hl = (self.gr.hl & 0xff00) | (v as u16),
            RegGHL::HL => bus.mem_write(self.mmu.to_physical(self.gr.hl), v),
            RegGHL::A => self.gr.a = v,
        }
    }
}

#[cfg(test)]
mod cpu_test {
    // TODO test addressing stuff
    #[test]
    fn signs() {
        // how does Rust handle "u16 as i32" conversion?
        let x: u16 = 0xffff;
        let y: i32 = x as i32;
        let d: i8 = -15;
        let z = (y + d as i32) as u16;
        assert_eq!(z, 0xfff0);
    }
}
