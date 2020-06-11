use std::num::Wrapping;

use crate::bus::Bus;
use crate::cpu::*;

/**
 * Stack and Exchange Instructions
 *
 * Table 44 of Z8018x specification.
 */

impl CPU {
    // All pushes are 16-bit.
    pub(super) fn push(&mut self, bus: &mut Bus, src: Operand) {
        let val = self.load_operand(bus, src);
        let sp = Wrapping(self.sr.sp);
        bus.mem_write(self.mmu.to_physical((sp - Wrapping(1)).0), (val >> 8) as u8);
        bus.mem_write(self.mmu.to_physical((sp - Wrapping(2)).0), val as u8);
        self.sr.sp = (sp - Wrapping(2)).0;
    }

    // All pops are also 16-bit.
    pub(super) fn pop(&mut self, bus: &mut Bus, dst: Operand) {
        let sp = Wrapping(self.sr.sp);
        let hi = bus.mem_read(self.mmu.to_physical(self.sr.sp)) as u16;
        let lo = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0)) as u16;
        self.store_operand(bus, dst, lo | hi << 8);
        self.sr.sp = (sp + Wrapping(2)).0;
    }
}
