use std::rc::Rc;

use crate::bus::Bus;
use crate::types::Peripheral;

// enums
pub mod enums;

// instruction set
mod alu;
mod alu16;
mod block;
mod ctrl;
mod iops;
mod ld_16bit;
mod ld_8bit;
mod rot;
mod special;
mod stack;

// instruction decode and dispatch
mod dispatch;

// peripherals
mod mmu;

pub use enums::*;

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
    ie: bool,
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
            ie: false,
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
    fn error(&mut self, cause: &str) {
        self.mode = Mode::Halt;
        self.sr.pc -= 1;
        println!("Illegal instruction (PC=${:04x}). Halt. {}", self.sr.pc, cause);
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
            Operand::Immediate() => self.error("store imm"),
            Operand::Immediate16() => self.error("store imm"),
            Operand::Relative() => self.error("store rel"),
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
}

#[cfg(test)]
mod cpu_test {
    use std::io::{stdout, Write};
    use std::rc::Rc;
    use std::{thread, time};

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
                stdout().flush().unwrap();
            }
        }
    }

    #[allow(dead_code)]
    fn print_cpu(cpu: &CPU, bus: &mut Bus) {
        let opcodes = [
            bus.mem_read(cpu.sr.pc as u32), // assume identity MMU
            bus.mem_read(cpu.sr.pc as u32 + 1),
            bus.mem_read(cpu.sr.pc as u32 + 2),
            bus.mem_read(cpu.sr.pc as u32 + 3),
            bus.mem_read(cpu.sr.pc as u32 + 4),
            bus.mem_read(cpu.sr.pc as u32 + 5),
        ];
        println!(
            "PC=${:04x}, opcode=${:02x} (0b{:08b}) \
                A=${:02x} BC=${:04x} DE=${:04x} HL=${:04x} \
                {}{}-{}-{}{}{}   {}",
            cpu.sr.pc,
            opcodes[0],
            opcodes[0],
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
            crate::disasm::disasm(&opcodes),
        );
    }

    #[allow(dead_code)]
    fn print_test(cpu: &CPU, bus: &mut Bus) {
        let opcodes = [
            bus.mem_read(0x1d42),
            bus.mem_read(0x1d43),
            bus.mem_read(0x1d44),
            bus.mem_read(0x1d45),
            bus.mem_read(0x0103),
            bus.mem_read(0x0104),
        ];
        println!(
            "IUIT={:02x} {:02x} {:02x} {:02x} MEM=${:02x}{:02x} \
                AF=${:02x}{:02x} BC=${:04x} DE=${:04x} HL=${:04x} IX=${:04x} IY=${:04x} SP=${:04x} \
                   {}",
            opcodes[0],
            opcodes[1],
            opcodes[2],
            opcodes[3],
            opcodes[4],
            opcodes[5],
            cpu.gr.a,
            cpu.gr.f,
            cpu.gr.bc,
            cpu.gr.de,
            cpu.gr.hl,
            cpu.sr.ix,
            cpu.sr.iy,
            cpu.sr.sp,
            crate::disasm::disasm(&opcodes),
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
                0xED, 0x19, 0xFF, //  OUTE:     OUT0 (0xff), e
                0xC9, //              DONE:     RET
                0x21, 0x00, 0x00, //  BOOT:     LD   hl,0
                0x36, 0x76, //                  LD   (hl),0x76 ; HALT
                0xC3, 0x00, 0x01, //            JP   0x100
            ],
        );
        ram.load_file(0x100, "tests/zexdoc.com")
            .expect("Loading ZEXDOC test binary");
        bus.add(ram.clone());

        let cio = CIO::new();
        bus.add(Rc::new(cio));

        // Run until HALT is executed
        cpu.reset();
        let mut loops = 0u64;
        let mut now = time::Instant::now();
        loop {
            cpu.cycle(&mut bus);
            if cpu.sr.pc == 0x122 {
                if loops > 0 {
                    println!("Test complete in {}s", now.elapsed().as_secs());
                    now = time::Instant::now();
                }
                // test completed
                loops = 0;
            }
            if cpu.sr.pc == 0x1d42 {
                println!("");
                print_test(&cpu, &mut bus);
            }
            if cpu.sr.pc == 0x1d46 {
                loops = loops + 1;
                if loops > 3 {
                    break;
                }
                print_test(&cpu, &mut bus);
            }
            // crc updated
            if cpu.sr.pc == 0x9d74 {
                let data = [
                    bus.mem_read(0x1e85),
                    bus.mem_read(0x1e86),
                    bus.mem_read(0x1e87),
                    bus.mem_read(0x1e88),
                ];
                print!("crc32 = ${:02x}{:02x}{:02x}{:02x}   ",
                        data[0], data[1], data[2], data[3]);
                print_cpu(&cpu, &mut bus);
            }
            // test loop
            if cpu.sr.pc == 0x1b27 {
                // loops = loops + 1;
            }
            // shifter fired
            if cpu.sr.pc == 0xfcad {
                let data = [
                    // iut
                    bus.mem_read(0x1d42),
                    bus.mem_read(0x1d43),
                    bus.mem_read(0x1d44),
                    bus.mem_read(0x1d45),
                    // msbt
                    bus.mem_read(0x103),
                    bus.mem_read(0x104),
                    bus.mem_read(0x105),
                    bus.mem_read(0x106),
                    bus.mem_read(0x107),
                    bus.mem_read(0x108),
                    bus.mem_read(0x109),
                    bus.mem_read(0x10a),
                    bus.mem_read(0x10b),
                    bus.mem_read(0x10c),
                    bus.mem_read(0x10d),
                    bus.mem_read(0x10e),
                    bus.mem_read(0x10f),
                    bus.mem_read(0x110),
                    bus.mem_read(0x111),
                    bus.mem_read(0x112),
                ];
                print!("#{:05}  ", loops);
                print!(
                    "uit=${:02x} ${:02x} ${:02x} ${:02x} ",
                    data[0], data[1], data[2], data[3],
                );
                print!(
                    "memop=${:02x}{:02x} iy=${:02x}{:02x} ix=${:02x}{:02x} hl=${:02x}{:02x} de=${:02x}{:02x} ",
                    data[4], data[5], data[6], data[7], data[8], data[9], data[10], data[11], data[12], data[13]
                );
                print!(
                    "bc=${:02x}{:02x} a=${:02x} f=${:02x} sp=${:02x}{:02x} ",
                    data[14], data[15], data[16], data[17], data[18], data[19]
                );
                println!("   {}", crate::disasm::disasm(&data));
                if data[10] == 0x39 && data[11] == 0xb3 {
                    thread::sleep(time::Duration::from_secs(1));
                }
            }
            if cpu.mode == crate::cpu::Mode::Halt {
                print_cpu(&cpu, &mut bus);
                break;
            }
        }
    }
}
