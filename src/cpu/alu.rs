use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 38 of Z8018x specification.
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
    pub(super) fn add_a_g(&mut self, bus: &mut Bus, g: u8) {
        match self.load_g_hl(bus, g) {
            Ok(reg) => {
                let result = self.gr.a as u16 + reg as u16;
                self.gr.f = CPU::add_flags(self.gr.a as u16, reg as u16, result);
                self.gr.a = result as u8;
                self.sr.pc += 1;
            }
            _ => self.error(),
        }
    }
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Flags, Register, CPU};
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
        assert_eq!(cpu.reg(Register::A), 112);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::B), 48);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 160); // 112+48=160
        assert_eq!(cpu.flags(), Flags::SF | Flags::VF); // Sz-h-Vnc: signed, overflowed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::C), 8);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 168); // 160+8=168
        assert_eq!(cpu.flags(), Flags::SF); // Sz-h-vnc: signed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 176); // 168+8=176
                                               //   0b1010_1000
                                               // + 0b0000_1000
                                               // = 0b1011_0000  (SF | HF)
        assert_eq!(cpu.flags(), Flags::SF | Flags::HF); // Sz-H-vnc: signed, half-carry

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 184); // 176+8=184
                                               //   0b1011_0000
                                               // + 0b0000_1000
                                               // = 0b1011_1000 (SF)
        assert_eq!(cpu.flags(), Flags::SF); // Sz-h-vnc: signed

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::HL), 0x000f);

        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 0); // 184+72=0
                                             //   0b1011_1000
                                             // + 0b0100_1000
                                             // = 0b0000_0000 (ZF, HF, CF)
        assert_eq!(cpu.flags(), Flags::ZF | Flags::HF | Flags::CF);
    }
}
