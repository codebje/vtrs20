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

    // Compute flags for 8-bit logical AND.
    // Set HF, reset NF, CF. Parity in PF. S, Z set based on result.
    fn and_flags(result: u16) -> u8 {
        let flags = (result & 0b1000_0000)                         // sf
            | (if (result & 0xff) == 0 { 0b0100_0000 } else { 0 }) // zf
            | ((!result.count_ones() as u16 & 1) << 2)             // pf
            | 0b0001_0000; // hf
        flags as u8
    }

    // Adder: used by all the src variations on ADD
    fn add(&mut self, src: u16, carry: u16) {
        let result = self.gr.a as u16 + src + carry;
        self.gr.f = CPU::add_flags(self.gr.a as u16, src, result);
        self.gr.a = result as u8;
        self.sr.pc += 1;
    }

    // Execute ADD A,g or ADD A,(HL)
    pub(super) fn add_a_ghl(&mut self, bus: &mut Bus, g: RegGHL) {
        let src = self.load_ghl(bus, g) as u16;
        self.add(src, 0);
    }

    // Execute ADD A,m
    pub(super) fn add_a_m(&mut self, bus: &mut Bus) {
        self.add(bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16, 0);
        self.sr.pc += 1; // consume 'm'
    }

    pub(super) fn adc_a_ghl(&mut self, bus: &mut Bus, g: RegGHL) {
        let src = self.load_ghl(bus, g) as u16;
        self.add(src, self.carry());
    }

    pub(super) fn adc_a_m(&mut self, bus: &mut Bus) {
        let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
        self.add(m, self.carry());
        self.sr.pc += 1;
    }

    // Execute INC g or INC (HL)
    pub(super) fn inc_ghl(&mut self, bus: &mut Bus, g: RegGHL) {
        let val = self.load_ghl(bus, g) as u16;
        let result = val + 1;

        // flags set like add, but preserve CF
        let flags = CPU::add_flags(val, 1, result) & 0b1111_1110;
        self.gr.f = flags | (self.gr.f & 0b1);

        self.store_ghl(bus, g, result as u8);

        self.sr.pc += 1;
    }

    // Logic: AND g, AND (HL)
    pub(super) fn and_a_ghl(&mut self, bus: &mut Bus, g: RegGHL) {
        let val = self.load_ghl(bus, g) as u16;
        let result = self.reg(Register::A) & val;

        self.gr.f = CPU::and_flags(result);
        self.gr.a = result as u8;

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

        // Register to test, value to expect, Optional flags to check
        let expected = [
            (Register::A, 112, None),
            (Register::B, 48, None),
            (Register::A, 160, Some(Flags::SF | Flags::VF)),
            (Register::C, 8, None),
            (Register::A, 168, Some(Flags::SF)),
            (Register::A, 176, Some(Flags::SF | Flags::HF)),
            (Register::A, 184, Some(Flags::SF)),
            (Register::HL, 15, None),
            (Register::A, 0, Some(Flags::ZF | Flags::HF | Flags::CF)),
        ];

        for (reg, val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(*reg), *val);
            match *flags {
                Some(f) => assert_eq!(cpu.flags(), f),
                None => (),
            }
        }
    }

    #[test]
    fn add_from_g() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x06, 0x14, //          0x0000  ld b, $14
                0x0e, 0x22, //          0x0002  ld c, $22
                0x16, 0x3f, //          0x0004  ld d, $3f
                0x1e, 0x4a, //          0x0006  ld e, $4a
                0x26, 0x89, //          0x0008  ld h, $89
                0x2e, 0x74, //          0x000a  ld l, $74
                0x36, 0xf2, //          0x000c  ld (hl), $f2
                0x3e, 0xbe, //          0x000e  ld a, $be
                0x80, //                0x0010  add a, b
                0x81, //                0x0011  add a, c
                0x82, //                0x0012  add a, d
                0x83, //                0x0013  add a, e
                0x84, //                0x0014  add a, h
                0x85, //                0x0015  add a, l
                0x86, //                0x0016  add a, (hl)
                0x87, //                0x0017  add a, a
                0x88, //                0x0018  adc a, b
                0x89, //                0x0019  adc a, c
                0x8a, //                0x0019  adc a, d
                0x8b, //                0x0019  adc a, e
                0x8c, //                0x0019  adc a, h
                0x8d, //                0x0019  adc a, l
                0x8e, //                0x0019  adc a, (hl)
                0x8f, //                0x0019  adc a, a
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        for _ in 0..8 {
            cpu.cycle(&mut bus)
        }

        let expected = [
            // add
            (0xd2, Flags::SF | Flags::HF),
            (0xf4, Flags::SF),
            (0x33, Flags::CF | Flags::HF),
            (0x7d, Flags::empty()),
            (0x06, Flags::CF | Flags::HF),
            (0x7a, Flags::empty()),
            (0x6c, Flags::CF),
            (0xd8, Flags::HF | Flags::VF | Flags::SF),
            // adc
            (0xec, Flags::SF),
            (0x0e, Flags::CF),
            (0x4e, Flags::HF), // has +1 from carry
            (0x98, Flags::SF | Flags::HF | Flags::VF),
            (0x21, Flags::HF | Flags::VF | Flags::CF),
            (0x96, Flags::SF | Flags::VF), // has +1 from carry
            (0x88, Flags::SF | Flags::CF),
            (0x11, Flags::HF | Flags::VF | Flags::CF), // has +1 from carry
        ];

        for (val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(Register::A), *val, "Testing value ${:02x}", *val);
            assert_eq!(cpu.flags(), *flags, "Testing value ${:02x}", *val);
        }
    }
    #[test]
    fn add_from_m() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x3e, 0xbe, //          0x0000  ld a, $be
                // add
                0xc6, 0x00, //          0x0002  add a, m
                0xc6, 0x08, //          0x0004  add a, m
                0xc6, 0x3a, //          0x0006  add a, m
                0xc6, 0x20, //          0x0008  add a, m
                0xc6, 0x70, //          0x000a  add a, m
                // adc
                0xce, 0x00, //          0x000c  add a, $00 ; just sign flag
                0xce, 0x70, //          0x000e  add a, $70 ; test 0x90+0x70 -> 0x00
                0xce, 0x00, //          0x0011  add a, $00 ; ensure carry is added
                0xce, 0x7f, //          0x0013  add a, $7f ; half-carry set without carry
                0xce, 0x8e, //          0x0015  add a, $8e ; set up 0x0e and carry
                0xce, 0x01, //          0x0017  add a, $01 ; half-carry set with carry
            ],
        );
        bus.add(ram.clone());
        cpu.reset();

        // load a known value into A
        cpu.cycle(&mut bus);

        let expected = [
            // add
            (0xbe, Flags::SF),
            (0xc6, Flags::SF | Flags::HF),
            (0x00, Flags::ZF | Flags::HF | Flags::CF),
            (0x20, Flags::empty()),
            (0x90, Flags::SF | Flags::VF),
            // adc
            (0x90, Flags::SF),
            (0x00, Flags::ZF | Flags::CF),
            (0x01, Flags::empty()),
            (0x80, Flags::SF | Flags::HF | Flags::VF),
            (0x0e, Flags::VF | Flags::CF),
            (0x10, Flags::HF),
        ];

        for (val, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(Register::A), *val, "Comparing A to ${:02x}", *val);
            assert_eq!(cpu.flags(), *flags, "Comparing A to ${:02x}", *val);
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
                0x06, 0x14, //          0x0000  ld b, $14
                0x0e, 0x99, //          0x0002  ld c, $99
                0x16, 0x3f, //          0x0004  ld d, $3f
                0x1e, 0x4a, //          0x0006  ld e, $4a
                0x26, 0x7f, //          0x0008  ld h, $7f
                0x2e, 0xff, //          0x000a  ld l, $ff
                0x36, 0xf2, //          0x000c  ld (hl), $f2
                0x3e, 0xbf, //          0x000e  ld a, $bf
                0x34, //                0x0010  inc (hl)
                0x04, //                0x0011  inc b
                0x0c, //                0x0012  inc c
                0x14, //                0x0013  inc d
                0x1c, //                0x0014  inc e
                0x24, //                0x0015  inc h
                0x2c, //                0x0016  inc l
                0x3c, //                0x0017  inc a
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
            assert_eq!(cpu.reg(*reg), *val);
            assert_eq!(cpu.flags(), *flags);
        }
    }

    #[test]
    pub fn and() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0xa0, //                and b
                0xa1, //                and c
                0xa2, //                and d
                0xa3, //                and e
                0xa4, //                and h
                0xa5, //                and l
                0xa6, //                and (hl)
                0xa7, //                and a
            ],
        );
        bus.add(ram.clone());
        cpu.reset();

        cpu.write_reg(Register::B, 0b1111_0000);
        cpu.write_reg(Register::C, 0b0000_1111);
        cpu.write_reg(Register::D, 0b0100_0001);
        cpu.write_reg(Register::E, 0b1000_0000);
        cpu.write_reg(Register::H, 0b0000_0000);
        cpu.write_reg(Register::L, 0b1111_1111);
        ram.write(0x00ff, &[0b10101010]);

        let expected = [
            (Register::B, 0b1011_0000, Flags::SF | Flags::HF),
            (Register::C, 0b0000_1110, Flags::HF),
            (Register::D, 0b0000_0000, Flags::ZF | Flags::HF | Flags::PF),
            (Register::E, 0b1000_0000, Flags::SF | Flags::HF),
            (Register::H, 0b0000_0000, Flags::ZF | Flags::HF | Flags::PF),
            (Register::L, 0b1011_1110, Flags::SF | Flags::HF | Flags::PF),
            (Register::HL, 0b1010_1010, Flags::SF | Flags::HF | Flags::PF),
            (Register::A, 0b1011_1110, Flags::SF | Flags::HF | Flags::PF),
        ];

        for (reg, val, flags) in &expected {
            println!("register {:?}", reg);
            cpu.write_reg(Register::A, 0b1011_1110);
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(Register::A), *val);
            assert_eq!(cpu.flags(), *flags);
        }
    }
}
