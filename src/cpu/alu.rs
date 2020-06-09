use crate::bus::Bus;
use crate::cpu::enums::*;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 40 of Z8018x specification.
 */

impl CPU {
    // Compute flags for 8-bit additions:
    //  Set Sign, Zero, Half, and Carry.
    //  Set P/V for oVerflow.
    //  Reset Negative.
    //
    // Overflow is set if the twos-complement addition is out of range. This happens only when
    // adding two positive numbers or two negative numbers (ie, bit 7 is equal in src and add)
    // and the result is a different sign.
    fn add_flags(src: u16, add: u16, result: u16) -> u8 {
        let flags =
            (result & 0b1000_0000)                  // sign equals bit 7 of result
            | (if (result & 0xff) == 0 { 0b0100_0000 } else { 0 }) // zero
            | ((src ^ add ^ result) & 0b0001_0000)  // half-carry set if bit carried into result bit 5
            | ((((src ^ add ^ 0b1000_0000) & (src ^ result)) >> 5) & 0b0000_0100) // overflow
            | (result >> 8 & 0b0000_0001)           // carry equals bit 8 of result
            ;
        flags as u8
    }

    // Execute ADD A,g or ADD A,(HL)
    pub(super) fn add_a_ghl(&mut self, bus: &mut Bus, g: RegGHL) {
        let reg = self.load_g_hl(bus, g);
        let result = self.gr.a as u16 + reg as u16;
        self.gr.f = CPU::add_flags(self.gr.a as u16, reg as u16, result);
        self.gr.a = result as u8;
        self.sr.pc += 1;
    }

    // Execute INC g or INC (HL)
    pub(super) fn inc_g_hl(&mut self, bus: &mut Bus, g: RegGHL) {
        let val = self.load_g_hl(bus, g);
        let val16 = val as u16;
        let result = val16 + 1;

        // flags set like add, but preserve CF
        let flags = CPU::add_flags(val16, 1, result) & 0b1111_1110;
        self.gr.f = flags | (self.gr.f & 0b1);

        self.store_g_hl(bus, g, result as u8);

        self.sr.pc += 1;
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Flags, Peripheral, Register, CPU};
    use crate::ram::RAM;
    use crate::rom::ROM;

    #[test]
    fn adder_flags() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let rom = ROM::new(
            0x80000,
            vec![
                0x3e, 0x70, //         0x0000   ld a, 112
                0x06, 0x30, //         0x0002   ld b, 48
                0x80, //               0x0004   add a, b
                0x0e, 0x08, //         0x0005   ld c, 8
                0x81, //               0x0007   add a, c
                0x81, //               0x0008   add a, c
                0x81, //               0x0009   add a, c
                0x21, 0x0f, 0x00, //   0x000a   ld hl, 0x000f
                0x86, //               0x000d   add a, (hl)
                0x06, 0x48, //         0x0010   ld b, 0x48
            ],
        );
        bus.add(Rc::new(rom));
        cpu.reset();

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 112);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::B), 48);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 160); // 112+48=160
        assert_eq!(cpu.flags(), Flags::SF | Flags::VF); // Sz-h-Vnc: signed, overflowed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::C), 8);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 168); // 160+8=168
        assert_eq!(cpu.flags(), Flags::SF); // Sz-h-vnc: signed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 176); // 168+8=176
                                                //   0b1010_1000
                                                // + 0b0000_1000
                                                // = 0b1011_0000  (SF | HF)
        assert_eq!(cpu.flags(), Flags::SF | Flags::HF); // Sz-H-vnc: signed, half-carry

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 184); // 176+8=184
                                                //   0b1011_0000
                                                // + 0b0000_1000
                                                // = 0b1011_1000 (SF)
        assert_eq!(cpu.flags(), Flags::SF); // Sz-h-vnc: signed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::HL), 0x000f);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(&Register::A), 0); // 184+72=0
                                              //   0b1011_1000
                                              // + 0b0100_1000
                                              // = 0b0000_0000 (ZF, HF, CF)
        assert_eq!(cpu.flags(), Flags::ZF | Flags::HF | Flags::CF);
    }

    #[test]
    fn add_from_g() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x06, 0x14, //          0x0002  ld b, $14
                0x0e, 0x22, //          0x0004  ld c, $22
                0x16, 0x3f, //          0x0006  ld d, $3f
                0x1e, 0x4a, //          0x0008  ld e, $4a
                0x26, 0x89, //          0x000a  ld h, $89
                0x2e, 0x74, //          0x000c  ld l, $74
                0x36, 0xf2, //          0x000e  ld (hl), $f2
                0x3e, 0xbe, //          0x0010  ld a, $be
                0x80, //                0x0012  add a, b
                0x81, //                0x0013  add a, c
                0x82, //                0x0014  add a, d
                0x83, //                0x0015  add a, e
                0x84, //                0x0016  add a, h
                0x85, //                0x0017  add a, l
                0x86, //                0x0018  add a, (hl)
                0x87, //                0x0019  add a, a
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..8 {
            cpu.cycle(&mut bus)
        }

        let expected = [
            (0xd2, Flags::SF | Flags::HF), //               0b1011_1110 + 0b0001_0100 = 0b1101_0010 SF HF
            (0xf4, Flags::SF), //                           0b1101_0010 + 0b0010_0010 = 0b1111_0100 SF
            (0x33, Flags::CF | Flags::HF), //               0b1111_0100 + 0b0011_1111 = 0b0011_0011 CF HF
            (0x7d, Flags::empty()), //                      0b0011_0011 + 0b0100_1010 = 0b0111_1101 -
            (0x06, Flags::CF | Flags::HF), //               0b0111_1101 + 0b1000_1001 = 0b0000_0110 CF HF
            (0x7a, Flags::empty()), //                      0b0000_0110 + 0b0111_0100 = 0b0111_1010 -
            (0x6c, Flags::CF), //                           0b0111_1010 + 0b1111_0010 = 0b0110_1100 CF
            (0xd8, Flags::HF | Flags::VF | Flags::SF), //   0b0110_1100 + 0b0110_1100 = 0b1101_1000 SF HF VF
        ];

        for (val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(&Register::A), *val);
            assert_eq!(cpu.flags(), *flags);
        }
    }

    #[test]
    fn inc() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x06, 0x14, //          0x0002  ld b, $14
                0x0e, 0x99, //          0x0004  ld c, $99
                0x16, 0x3f, //          0x0006  ld d, $3f
                0x1e, 0x4a, //          0x0008  ld e, $4a
                0x26, 0x7f, //          0x000a  ld h, $7f
                0x2e, 0xff, //          0x000c  ld l, $ff
                0x36, 0xf2, //          0x000e  ld (hl), $f2
                0x3e, 0xbf, //          0x0010  ld a, $bf
                0x34, //                0x0018  inc (hl)
                0x04, //                0x0012  inc b
                0x0c, //                0x0013  inc c
                0x14, //                0x0014  inc d
                0x1c, //                0x0015  inc e
                0x24, //                0x0016  inc h
                0x2c, //                0x0017  inc l
                0x3c, //                0x0019  inc a
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..8 {
            cpu.cycle(&mut bus)
        }

        // Hit up (HL) first so H, L don't get changed
        cpu.cycle(&mut bus);
        assert_eq!(ram.mem_read(0x7fff), Some(0xf3));
        assert_eq!(cpu.flags(), Flags::SF);

        let expected = [
            (Register::B, 0x15, Flags::empty()),
            (Register::C, 0x9a, Flags::SF),
            (Register::D, 0x40, Flags::HF),
            (Register::E, 0x4b, Flags::empty()),
            (Register::H, 0x80, Flags::VF | Flags::SF | Flags::HF),
            (Register::L, 0x00, Flags::ZF | Flags::HF),
            (Register::A, 0xc0, Flags::HF | Flags::SF),
        ];

        for (reg, val, flags) in &expected {
            println!("register {:?}", reg);
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(reg), *val);
            assert_eq!(cpu.flags(), *flags);
        }
    }
}
