use crate::cpu::enums::*;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    pub(super) fn add_hl_ww(&mut self, ww: RegW, with_carry: bool) {
        let carry = if with_carry { self.gr.f & Flags::CF.bits() } else { 0 };
        let value = self.reg(ww) as u32;
        let hl = self.gr.hl as u32;
        let result = hl + value + carry as u32;

        // For no reason ADC HL, ww sets SF, ZF, V; ADD HL, ww does not.
        if with_carry {
            let signs = (self.gr.hl ^ value as u16 ^ 0b1000_0000_0000_0000) & (self.gr.hl ^ result as u16);
            self.gr.f = ((result & 0b1000_0000_0000_0000) >> 8) as u8
                | (if (result & 0xffff) == 0 { 0b0100_0000 } else { 0 }) as u8
                | ((signs >> 13) & 0b0000_0100) as u8
                | ((result >> 16) & 1) as u8;
        } else {
            // Half-carry: undefined; Negative: reset; Carry: modified; Others: unchanged
            self.gr.f = (self.gr.f & 0b1111_1100) | ((result >> 16) & 0b0000_0001) as u8;
        }

        self.gr.hl = result as u16;
    }

    pub(super) fn sub_hl_ww(&mut self, ww: RegW, borrow: bool) {
        let carry = if borrow { self.gr.f & Flags::CF.bits() } else { 0 };
        let operand = self.reg(ww) as i32;
        let result = self.gr.hl as i32 - operand - carry as i32;

        // sign flag set if bit 16 set
        // zero flag set if result is zero
        // overflow flag set if pos+pos=neg or neg+neg=pos
        // negative flag always set
        // carry flag set if result is negative
        let signs = (self.gr.hl ^ operand as u16 ^ 0b1000_0000_0000_0000) & (self.gr.hl ^ result as u16);
        self.gr.f = (result & 0b1000_0000_0000_0000 >> 8) as u8
            | (if (result & 0xffff) == 0 { 0b0100_0000 } else { 0 }) as u8
            | ((signs >> 13) & 0b0000_0100) as u8
            | Flags::NF.bits()
            | if result < 0 { Flags::CF.bits() } else { 0 };
        self.gr.hl = result as u16;
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
                0x01, 0x99, 0x14, //    ld bc, $1499
                0x11, 0xff, 0x3f, //    ld de, $3fff
                0x21, 0xff, 0x7f, //    ld hl, $7fff
                0x31, 0x11, 0x11, //    ld sp, $1111
                0x09, //                add hl, bc
                0x19, //                add hl, de
                0x29, //                add hl, hl
                0x39, //                add hl, sp
                0xed, 0x4a, //          adc hl, bc
                0xed, 0x5a, //          adc hl, de
                0xed, 0x6a, //          adc hl, hl
                0xed, 0x7a, //          adc hl, sp
                0xed, 0x5a, //          adc hl, de
                0xed, 0x5a, //          adc hl, de
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..4 {
            cpu.cycle(&mut bus)
        }

        let expected = [
            ("ADD", Register::BC, 0x9498, Flags::empty()),
            ("ADD", Register::DE, 0xd497, Flags::empty()),
            ("ADD", Register::HL, 0xa92e, Flags::CF),
            ("ADD", Register::SP, 0xba3f, Flags::empty()),
            ("ADC", Register::BC, 0xced8, Flags::SF),
            ("ADC", Register::DE, 0x0ed7, Flags::CF),
            ("ADC", Register::HL, 0x1daf, Flags::empty()),
            ("ADC", Register::SP, 0x2ec0, Flags::empty()),
            ("ADC", Register::BC, 0x6ebf, Flags::empty()),
            ("ADC", Register::BC, 0xaebe, Flags::SF | Flags::VF),
        ];

        for (op, reg, val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.gr.hl, *val, "{} HL, {}", *op, *reg);
            assert_eq!(cpu.flags(), *flags, "{} HL, {}", *op, *reg);
        }
    }
}
