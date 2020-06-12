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
    pub(super) fn ldir(&mut self, bus: &mut Bus) {
        let mut hl = Wrapping(self.gr.hl);
        let mut de = Wrapping(self.gr.de);
        let mut bc = Wrapping(self.gr.bc);
        let one = Wrapping(1u16);

        loop {
            let val = bus.mem_read(self.mmu.to_physical(hl.0));
            bus.mem_write(self.mmu.to_physical(de.0), val);
            de = de + one;
            hl = hl + one;
            bc = bc - one;
            if bc.0 == 0 {
                break;
            }
        }

        self.gr.hl = hl.0;
        self.gr.de = de.0;
        self.gr.bc = bc.0;

        self.gr.f &= !0b0001_0110;
    }
}
