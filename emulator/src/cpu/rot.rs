use crate::bus::Bus;
use crate::cpu::enums::*;
use crate::cpu::{CheckFlags, CPU};

/**
 * Rotate and Shift Instructions
 *
 * Table 39 of Z8018x specification.
 */

impl CPU {
    pub(super) fn rot_left(&mut self, bus: &mut Bus, operand: Operand, op: ShiftOp, mode: ShiftMode) {
        let mut src = self.load_operand(bus, operand) << 1;

        if op == ShiftOp::ShiftL {
            self.warn("Illegal instruction: shift left logical");
        }

        match op {
            ShiftOp::Rot => src |= (self.gr.f & 0b1) as u16,
            ShiftOp::RotC => src |= src >> 8,
            ShiftOp::ShiftA => (),
            ShiftOp::ShiftL => src |= 1,
        }

        match mode {
            // 8080 rotates preserve S, Z, and P/V, reset H and N, and modify C.
            ShiftMode::R8080 => self.gr.f = self.gr.f & 0b1100_0100,

            // Z80 rotates modify S, Z, P, and C, and reset H and N.
            ShiftMode::RZ80 => self.gr.f = src.sign() | src.zero() | src.parity(),
        }
        self.gr.f |= (src >> 8) as u8;

        self.store_operand(bus, operand, src);
    }

    pub(super) fn rot_right(&mut self, bus: &mut Bus, operand: Operand, op: ShiftOp, mode: ShiftMode) {
        let src = self.load_operand(bus, operand);
        let mut result = src >> 1;

        match op {
            ShiftOp::Rot => result |= ((self.gr.f & 0b1) << 7) as u16,
            ShiftOp::RotC => result |= src << 7,
            ShiftOp::ShiftA => result |= src & 0b1000_0000,
            ShiftOp::ShiftL => (),
        }

        match mode {
            // 8080 rotates preserve S, Z, and P/V, reset H and N, and modify C.
            ShiftMode::R8080 => self.gr.f = self.gr.f & 0b1100_0100,

            // Z80 rotates modify S, Z, P, and C, and reset H and N.
            ShiftMode::RZ80 => self.gr.f = result.sign() | result.zero() | result.parity(),
        }
        self.gr.f |= (src as u8) & 0b0000_0001;

        self.store_operand(bus, operand, result);
    }

    // low nibble of (hl) moves to high nibble of (hl)
    // high nibble of (hl) moves to low nibble of a
    // low nibble of a moves to low nibble of (hl)
    pub(super) fn rld(&mut self, bus: &mut Bus) {
        let hl = bus.mem_read(self.mmu.to_physical(self.gr.hl));
        let a = self.gr.a & 0xf0 | ((hl & 0xf0) >> 4);
        let result = ((hl & 0x0f) << 4) | self.gr.a & 0x0f;
        bus.mem_write(self.mmu.to_physical(self.gr.hl), result);
        self.gr.a = a;
        // TODO the z180 sets flags based on result, not a, but the z80 sets flags based on a
        self.gr.f = a.sign() | a.zero() | a.parity() | self.gr.f & 1;
    }

    // low nibble of a moves to high nibble of (hl)
    // high nibble of (hl) moves to low nibble of (hl)
    // low nibble of (hl) moves to a
    pub(super) fn rrd(&mut self, bus: &mut Bus) {
        let hl = bus.mem_read(self.mmu.to_physical(self.gr.hl));
        let a = self.gr.a & 0xf0 | (hl & 0x0f);
        let result = ((hl & 0xf0) >> 4) | ((self.gr.a & 0x0f) << 4);
        bus.mem_write(self.mmu.to_physical(self.gr.hl), result);
        self.gr.a = a;
        // TODO the z180 sets flags based on result, not a, but the z80 sets flags based on a
        self.gr.f = a.sign() | a.zero() | a.parity() | self.gr.f & 1;
    }

    pub(super) fn bit(&mut self, bus: &mut Bus, bit: u8, src: Operand) {
        let data = self.load_operand(bus, src) as u8;

        self.gr.f |= 0b0001_0000; // half-carry is set
        self.gr.f &= 0b1011_1101; // negative is reset, clear zero
        self.gr.f |= ((!data >> bit) & 1) << 6;
    }

    pub(super) fn set(&mut self, bus: &mut Bus, bit: u8, src: Operand) {
        let data = self.load_operand(bus, src);
        self.store_operand(bus, src, data | (1 << bit));
    }

    pub(super) fn res(&mut self, bus: &mut Bus, bit: u8, src: Operand) {
        let data = self.load_operand(bus, src);
        self.store_operand(bus, src, data & !(1 << bit));
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
