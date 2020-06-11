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
mod stack;

// instruction decode and dispatch
mod dispatch;

// peripherals
mod mmu;

use enums::*;

#[derive(Debug, PartialEq)]
pub enum Mode {
    Reset,
    OpCodeFetch,
    //IntAckNMI,
    //IntAckINT0,
    //IntAckOther,
    //BusRelease,
    Halt,
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
    const FLAG_C: u8 = 0b0000_0001;
    const FLAG_P: u8 = 0b0000_0100;
    const FLAG_Z: u8 = 0b0100_0000;
    const FLAG_S: u8 = 0b1000_0000;

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
            Register::AF => (self.gr.a as u16) << 8 | self.gr.f as u16,
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
            Register::AF => {
                self.gr.a = (v >> 8) as u8;
                self.gr.f = v as u8;
            }
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
            Mode::Halt => (),
        }
    }

    // enter an error state
    fn error(&mut self) {
        self.mode = Mode::Halt;
        println!("Illegal instruction (PC=${:04x}). Halt.", self.sr.pc);
    }

    // Load an operand using an addressing mode. Will adjust PC as needed.
    fn load_operand(&mut self, bus: &mut Bus, operand: Operand) -> u16 {
        match operand {
            Operand::Direct(reg) => self.reg(reg),
            Operand::Indirect(reg) => bus.mem_read(self.mmu.to_physical(self.reg(reg))) as u16,
            Operand::Indexed(reg) => {
                let d = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as i8;
                let addr = self.reg(reg) as i32 + d as i32;
                self.sr.pc += 1;
                bus.mem_read(self.mmu.to_physical(addr as u16)) as u16
            }
            Operand::Extended() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                bus.mem_read(self.mmu.to_physical(addr)) as u16
            }
            Operand::Extended16() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                let lo = bus.mem_read(self.mmu.to_physical(addr)) as u16;
                let hi = bus.mem_read(self.mmu.to_physical(addr + 1)) as u16;
                hi << 8 | lo
            }
            Operand::Immediate() => {
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                self.sr.pc += 1;
                m
            }
            Operand::Immediate16() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                addr
            }
            Operand::Relative() => {
                let d = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as i8;
                self.sr.pc += 1;
                (self.sr.pc as i32 + d as i32) as u16
            }
        }
    }

    // Store a result into an operand. This will halt the CPU on Immediate or Relative.
    fn store_operand(&mut self, bus: &mut Bus, operand: Operand, value: u16) {
        match operand {
            Operand::Direct(reg) => self.write_reg(reg, value),
            Operand::Indirect(reg) => bus.mem_write(self.mmu.to_physical(self.reg(reg)), value as u8),
            Operand::Indexed(reg) => {
                let d = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as i8;
                let addr = self.reg(reg) as i32 + d as i32;
                self.sr.pc += 1;
                bus.mem_write(self.mmu.to_physical(addr as u16), value as u8);
            }
            Operand::Extended() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                bus.mem_write(self.mmu.to_physical(addr), value as u8);
            }
            Operand::Extended16() => {
                let n = bus.mem_read(self.mmu.to_physical(self.sr.pc)) as u16;
                let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
                let addr = m << 8 | n;
                self.sr.pc += 2;
                bus.mem_write(self.mmu.to_physical(addr), value as u8);
                bus.mem_write(self.mmu.to_physical(addr + 1), (value >> 8) as u8);
            }
            Operand::Immediate() => self.error(),
            Operand::Immediate16() => self.error(),
            Operand::Relative() => self.error(),
        }
    }

    fn is_condition(&self, condition: Option<Condition>) -> bool {
        match condition {
            Some(Condition::NonZero) => self.gr.f & CPU::FLAG_Z == 0,
            Some(Condition::Zero) => self.gr.f & CPU::FLAG_Z != 0,
            Some(Condition::NonCarry) => self.gr.f & CPU::FLAG_C == 0,
            Some(Condition::Carry) => self.gr.f & CPU::FLAG_C != 0,
            Some(Condition::ParityOdd) => self.gr.f & CPU::FLAG_P == 0,
            Some(Condition::ParityEven) => self.gr.f & CPU::FLAG_P != 0,
            Some(Condition::SignPlus) => self.gr.f & CPU::FLAG_S == 0,
            Some(Condition::SignMinus) => self.gr.f & CPU::FLAG_S != 0,
            None => true,
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
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Peripheral, CPU};
    use crate::ram::RAM;

    struct CIO {}

    impl CIO {
        fn new() -> CIO {
            CIO {}
        }
    }

    impl Peripheral for CIO {
        fn io_write(&self, address: u16, data: u8) {
            if address == 0xff {
                print!("{}", data as char);
            }
        }
    }

    #[allow(dead_code)]
    fn print_cpu(cpu: &CPU, bus: &mut Bus) {
        let opcode = bus.mem_read(cpu.sr.pc as u32); // assume identity MMU
        println!(
            "PC=${:04x}, opcode=${:02x} (0b{:08b}) \
                A=${:02x} BC=${:04x} DE=${:04x} HL=${:04x} \
                {}{}-{}-{}{}{}",
            cpu.sr.pc,
            opcode,
            opcode,
            cpu.gr.a,
            cpu.gr.bc,
            cpu.gr.de,
            cpu.gr.hl,
            if cpu.gr.f & 0b1000_0000 != 0 { 'S' } else { 's' },
            if cpu.gr.f & 0b0100_0000 != 0 { 'Z' } else { 'z' },
            if cpu.gr.f & 0b0001_0000 != 0 { 'H' } else { 'h' },
            if cpu.gr.f & 0b0000_0100 != 0 { 'P' } else { 'p' },
            if cpu.gr.f & 0b0000_0010 != 0 { 'N' } else { 'n' },
            if cpu.gr.f & 0b0000_0001 != 0 { 'C' } else { 'c' },
        );
    }

    #[test]
    fn zexdoc() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        // CPM control
        ram.write(
            0x0,
            &[
                0xC3, 0x1E, 0x00, //            JP   boot
                0x00, //                        NOP
                0x00, //                        NOP
                0x3E, 0x02, //        CPM:      LD   a,2
                0xB9, //                        CP   c
                0xCA, 0x1A, 0x00, //            JP   z,oute
                0x62, 0x6B, //                  LD   hl,de
                0x7E, //              LOOP:     LD   a,(hl)
                0xFE, 0x24, //                  CP   '$'
                0xCA, 0x1D, 0x00, //            JP   z,done
                0xED, 0x39, 0xFF, //            OUT0 (0xff), a
                0x23, //                        INC   hl
                0xC3, 0x0D, 0x00, //            JP   loop
                0xED, 0x09, 0xFF, //  OUTE:     OUT0 (0xff), c
                0xC9, //              DONE:     RET
                0x21, 0x00, 0x00, //  BOOT:     LD   hl,0
                0x36, 0x76, //                  LD   (hl),0x76 ; HALT
                0xC3, 0x00, 0x01, //            JP   0x100
            ],
        );
        ram.load_file(0x100, "test/zexdoc.com")
            .expect("Loading ZEXDOC test binary");
        bus.add(ram.clone());

        let cio = CIO::new();
        bus.add(Rc::new(cio));

        // Run until HALT is executed
        cpu.reset();
        loop {
            print_cpu(&cpu, &mut bus);
            cpu.cycle(&mut bus);
            if cpu.mode == crate::cpu::Mode::Halt {
                break;
            }
        }
    }
}
