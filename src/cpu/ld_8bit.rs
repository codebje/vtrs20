use crate::bus::Bus;
use crate::cpu::*;

/**
 * 8-bit Load
 *
 * Table 41 of Z8018x specification.
 */

impl CPU {
    // There's really nothing to this after the decode...
    pub(super) fn ld_8(&mut self, bus: &mut Bus, src: Operand, dst: Operand) {
        let val = self.load_operand(bus, src);
        self.store_operand(bus, dst, val);
    }

    // Execute LD g, m or LD (HL), m
    pub(super) fn ld_ghl_m(&mut self, bus: &mut Bus, g: RegGHL) {
        let imm = bus.mem_read(self.mmu.to_physical(self.sr.pc));
        self.store_ghl(bus, g, imm);
        self.sr.pc += 1;
    }

    // Execute LD (regpair), A
    // This instruction is only defined for BC, DE but this code will work
    // for _any_ register, 8 or 16-bit. This is unchecked during execution.
    pub(super) fn ld_indirect_a(&mut self, bus: &mut Bus, reg: Register) {
        bus.mem_write(self.mmu.to_physical(self.reg(reg)), self.gr.a);
    }
}

#[cfg(test)]
mod ld_test {
    use std::rc::Rc;

    use crate::bus::Bus;
    use crate::cpu::{Register, CPU};
    use crate::ram::RAM;
    use crate::types::Peripheral;

    #[test]
    fn load_immediate() {
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
            ],
        );
        bus.add(ram.clone());
        cpu.reset();
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        assert_eq!(cpu.reg(Register::A), 0xbe);
        assert_eq!(cpu.reg(Register::BC), 0x1422);
        assert_eq!(cpu.reg(Register::DE), 0x3f4a);
        assert_eq!(cpu.reg(Register::HL), 0x8974);
        assert_eq!(ram.mem_read(0x8974), Some(0xf2));
    }

    #[test]
    fn load_indirect() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        ram.write(
            0x0000,
            &[
                0x06, 0x7e, //          0x0002  ld b, $7e
                0x0e, 0xaf, //          0x0004  ld c, $af
                0x3e, 0xbe, //          0x0006  ld a, $be
                0x02, //                0x0008  ld (bc), a
                0x16, 0x3f, //          0x0009  ld d, $3f
                0x1e, 0x4a, //          0x000b  ld e, $4a
                0x12, //                0x000d  ld (de), a
            ],
        );
        bus.add(ram.clone());
        cpu.reset();

        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        assert_eq!(ram.mem_read(0x7eaf), Some(0xbe));

        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        cpu.cycle(&mut bus);
        assert_eq!(ram.mem_read(0x3f4a), Some(0xbe));
    }
}
