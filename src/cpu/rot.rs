use crate::bus::Bus;
use crate::cpu::enums::*;
use crate::cpu::CPU;

/**
 * Rotate and Shift Instructions
 *
 * Table 39 of Z8018x specification.
 */

impl CPU {
    pub(super) fn rot_left(&mut self, bus: &mut Bus, operand: Operand, carry: bool) {
        let mut src = self.load_operand(bus, operand) << 1;
        if carry {
            // RLCA sets bit 0 to bit 8
            src |= src >> 8;
        } else {
            // RLA sets bit 0 to CF
            src |= (self.gr.f & 0b1) as u16;
        }
        self.gr.f = (self.gr.f & 0b1110_1100) | (src >> 8) as u8;
        self.store_operand(bus, operand, src);
    }

    pub(super) fn rot_right(&mut self, bus: &mut Bus, operand: Operand, carry: bool) {
        let src = self.load_operand(bus, operand);
        let mut result = src >> 1;
        if carry {
            // RRCA sets bit 7 to bit 0
            result |= src << 7;
        } else {
            // RRA sets bit 7 to CF
            result |= (self.gr.f & 0b1 << 7) as u16;
        }
        self.gr.f = (self.gr.f & 0b1110_1100) | (src & 1) as u8;
        self.store_operand(bus, operand, result);
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Flags, Register, CPU};
    use crate::ram::RAM;

    #[test]
    fn rot_left() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x3e, 0x70, //          ld a, 112
                0x17, //                rla
                0x17, //                rla
                0x07, //                rlca
                0x07, //                rlca
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        cpu.cycle(&mut bus);

        let expected = [
            (0xe0, "rla 0x70 = 0xe0", Flags::empty()),
            (0xc0, "rla 0x70 = 0xc0, carry set", Flags::CF),
            (0x81, "rlca 0xc0 = 0x81, carry set", Flags::CF),
            (0x03, "rlca 0x81 = 0x35, carry set", Flags::CF),
        ];

        for (val, msg, flags) in &expected {
            cpu.cycle(&mut bus);
            assert_eq!(cpu.reg(Register::A), *val, "{}", msg);
            assert_eq!(cpu.flags(), *flags, "{}", msg);
        }
    }
}
