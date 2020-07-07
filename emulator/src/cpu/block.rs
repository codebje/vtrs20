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
        let cf = self.gr.f & 1;

        // Perform A-(HL), discarding the result value. Flags are updated.
        self.sub_a(bus, Operand::Indirect(RegIndirect::HL), false, false);

        // Restore the preserved carry flag, reset parity/overlow flag
        self.gr.f = (self.gr.f & 0b1101_0010) | cf;

        // decrement hl, bc
        self.gr.hl = (Wrapping(self.gr.hl) - Wrapping(1)).0;
        self.gr.bc = (Wrapping(self.gr.bc) - Wrapping(1)).0;

        // set PF if BC is not zero
        if self.gr.bc != 0 {
            self.gr.f |= Flags::PF.bits();
        }

        // if bc not zero and a != (hl), CPDR will decrement PC
        if repeating && self.gr.bc != 0 && (self.gr.f & Flags::ZF.bits()) == 0 {
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
