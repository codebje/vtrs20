use crate::bus::Bus;
use crate::cpu::enums::*;
use crate::cpu::CPU;

/**
 * Arithmetic and Logical Functions (8-bit)
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // Execute INC ww
    pub(super) fn inc_ww(&mut self, bus: &mut Bus, ww: RegW) {}
}

#[cfg(test)]
mod alu_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Flags, Peripheral, Register, CPU};
    use crate::ram::RAM;

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
