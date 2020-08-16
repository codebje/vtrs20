use crate::cpu::enums::*;
use crate::cpu::{CheckFlags, CPU};

/**
 * Special Control Instructions
 *
 * Table 47 of Z8018x specification.
 */

impl CPU {
    pub(super) fn nop(&mut self) {}

    pub(super) fn scf(&mut self) {
        self.gr.f = (self.gr.f & 0b1100_0101) | 0b0000_0001;
    }

    pub(super) fn ccf(&mut self) {
        // note: the Z80 copies C into H, while the Z180 resets H
        // todo: correct this to z180 behaviour after confirming a corrected CRC for zexdoc's daaop test
        self.gr.f = ((self.gr.f & 0b1100_0101) ^ 0b0000_0001) | (self.gr.f & 0b0000_0001) << 4;
    }

    // This is a Z80-style daa, because the Z180 user manual doesn't really specify how it works.
    pub(super) fn daa(&mut self) {
        let mut result = self.gr.a as u16;
        let mut flags = self.gr.f & Flags::NF.bits();

        if self.gr.a & 0xf > 9 || (self.gr.f & Flags::HF.bits()) != 0 {
            if self.gr.f & Flags::NF.bits() == 0 {
                result += 6;
                if self.gr.a & 0x0f > 9 {
                    flags |= Flags::HF.bits();
                }
            } else {
                result += 0xfa;
                if self.gr.a & 0xf < 6 && self.gr.f & Flags::HF.bits() != 0 {
                    flags |= Flags::HF.bits();
                }
            }
        }

        if self.gr.f & Flags::CF.bits() != 0 || self.gr.a & 0xf0 > 0x90 || (self.gr.a & 0xf0 == 0x90 && self.gr.a & 0xf > 9)
        {
            result += if (self.gr.f & Flags::NF.bits()) == 0 { 0x60 } else { 0xa0 };
            flags |= Flags::CF.bits();
        }

        self.gr.a = result as u8;
        self.gr.f = flags | result.sign() | result.zero() | result.parity();
    }

    pub(super) fn di(&mut self) {
        self.ief1 = false;
        self.ief2 = false;
    }

    pub(super) fn ei(&mut self) {
        self.ief1 = true;
        self.ief2 = true;
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Register, CPU};
    use crate::ram::RAM;

    #[test]
    fn nop_incrs_pc() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(0x0000, &[0x00]);
        bus.add(ram);
        cpu.reset();

        let pc_prev = cpu.reg(Register::PC);
        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::PC), pc_prev + 1, "PC increments");
    }

    #[test]
    fn daa() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(0x0000, &[0xaf, 0x3d, 0x27]);
        bus.add(ram);
        cpu.reset();

        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 0xf9);
    }
}
