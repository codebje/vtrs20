use crate::cpu::enums::*;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // Execute INC ww
    pub(super) fn inc_ww(&mut self, ww: RegW) {
        match ww {
            RegW::BC => self.gr.bc = (self.gr.bc as u32 + 1) as u16,
            RegW::DE => self.gr.de = (self.gr.de as u32 + 1) as u16,
            RegW::HL => self.gr.hl = (self.gr.hl as u32 + 1) as u16,
            RegW::SP => self.sr.sp = (self.sr.sp as u32 + 1) as u16,
        }
        self.sr.pc += 1;
    }

    pub(super) fn add_hl_ww(&mut self, ww: RegW) {
        let value = self.reg(ww) as u32;
        let hl = self.gr.hl as u32;
        let result = hl + value;
        self.gr.hl = result as u16;

        // Half-carry: undefined; Negative: reset; Carry: modified; Others: unchanged
        self.gr.f = (self.gr.f & 0b1111_1100) | ((result >> 16) & 0b0000_0001) as u8;

        self.sr.pc += 1;
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::*;
    use crate::ram::RAM;

    #[test]
    fn inc() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x01, 0x99, 0x14, //    0x0000  ld bc, $1499
                0x11, 0xff, 0xff, //    0x0003  ld de, $ffff
                0x21, 0xff, 0x7f, //    0x0006  ld hl, $7fff
                0x03, //                0x000e  inc bc
                0x13, //                0x000f  inc de
                0x23, //                0x0010  inc hl
                0x33, //                0x0011  inc sp
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..3 {
            cpu.cycle(&mut bus)
        }

        let expected = [
            (Register::BC, 0x149a),
            (Register::DE, 0x0000),
            (Register::HL, 0x8000),
            (Register::SP, 0x0001),
        ];

        for (reg, val) in &expected {
            println!("register {:?}", reg);
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(*reg), *val);
        }
    }

    #[test]
    fn add_hl_ww() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x01, 0x99, 0x14, //    0x0000  ld bc, $1499
                0x11, 0xff, 0x3f, //    0x0003  ld de, $3fff
                0x21, 0xff, 0x7f, //    0x0006  ld hl, $7fff
                0x31, 0x11, 0x11, //    0x0009  ld sp, $1111
                0x09, //                0x000c  add hl, bc
                0x19, //                0x000d  add hl, de
                0x29, //                0x000e  add hl, hl
                0x39, //                0x000f  add hl, sp
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..4 {
            cpu.cycle(&mut bus)
        }

        let expected = [
            (Register::BC, 0x9498, Flags::empty()),
            (Register::DE, 0xd497, Flags::empty()),
            (Register::HL, 0xa92e, Flags::CF),
            (Register::SP, 0xba3f, Flags::empty()),
        ];

        for (reg, val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.gr.hl, *val, "ADD HL, {}", *reg);
            assert_eq!(cpu.flags(), *flags);
        }
    }
}
