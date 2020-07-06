use std::num::Wrapping;

use crate::bus::Bus;
use crate::cpu::*;

/**
 * Block Transfer
 *
 * Table 43 of Z8018x specification.
 */

impl CPU {
    // Transfer (HL)m -> (DE)m
    // Subtract 1 from BC, add 1 to HL, DE
    // Stop when BC == 0
    pub(super) fn ldi(&mut self, bus: &mut Bus, repeating: bool) {
        let byte = self.load_operand(bus, Operand::Indirect(RegIndirect::HL));
        self.store_operand(bus, Operand::Indirect(RegIndirect::DE), byte);

        self.gr.hl = (Wrapping(self.gr.hl) + Wrapping(1)).0;
        self.gr.bc = (Wrapping(self.gr.bc) - Wrapping(1)).0;
        self.gr.de = (Wrapping(self.gr.de) + Wrapping(1)).0;
        self.gr.f &= 0b1100_0001;

        if repeating && self.gr.bc != 0 {
            self.sr.pc -= 2;
        }

        if !repeating && self.gr.bc == 1 {
            self.gr.f |= 0b0000_0100;
        }
    }

    pub(super) fn cpd(&mut self, bus: &mut Bus, repeating: bool) {
        let hl = Wrapping(self.load_operand(bus, Operand::Indirect(RegIndirect::HL)) as u8);
        let r = (Wrapping(self.gr.a) - hl).0;

        self.gr.f &= Flags::CF.bits(); // preserve CF
        self.gr.f |= Flags::NF.bits(); // set NF
        self.gr.f |= if (r & 0x80) != 0 { Flags::SF.bits() } else { 0 }; // check SF
        self.gr.f |= if r == 0 { Flags::ZF.bits() } else { 0 }; // check ZF
        self.gr.f |= (r ^ hl.0 ^ self.gr.a) & Flags::HF.bits();

        if self.gr.bc != 1 {
            self.gr.f |= 0b0000_0100;
        }

        // decrement hl, bc
        self.gr.hl = (Wrapping(self.gr.hl) - Wrapping(1)).0;
        self.gr.bc = (Wrapping(self.gr.bc) - Wrapping(1)).0;

        // if bc not zero and a != (hl), CPDR will decrement PC
        if repeating && self.gr.bc != 0 && hl.0 != self.gr.a {
            self.sr.pc -= 2;
        }
    }

    pub(super) fn ldd(&mut self, bus: &mut Bus, repeating: bool) {
        let byte = self.load_operand(bus, Operand::Indirect(RegIndirect::HL));
        self.store_operand(bus, Operand::Indirect(RegIndirect::DE), byte);

        self.gr.hl = (Wrapping(self.gr.hl) - Wrapping(1)).0;
        self.gr.bc = (Wrapping(self.gr.bc) - Wrapping(1)).0;
        self.gr.de = (Wrapping(self.gr.de) - Wrapping(1)).0;
        self.gr.f &= 0b1100_0001;

        if repeating && self.gr.bc != 0 {
            self.sr.pc -= 2;
        }

        if !repeating && self.gr.bc == 1 {
            self.gr.f |= 0b0000_0100;
        }
    }
}
