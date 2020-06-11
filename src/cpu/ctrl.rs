use std::num::Wrapping;

use crate::bus::Bus;
use crate::cpu::*;

/**
 * Program Control Instructions
 *
 * Table 45 of Z8018x specification.
 */

impl CPU {
    // JP mn
    pub(super) fn jp(&mut self, bus: &mut Bus, src: Operand, condition: Option<Condition>) {
        let dest = self.load_operand(bus, src);
        if self.is_condition(condition) {
            self.sr.pc = dest;
        }
    }

    pub(super) fn call(&mut self, bus: &mut Bus, src: Operand, condition: Option<Condition>) {
        // PCHr -> (SP-1)m
        // PCLr -> (SP-2)m
        // mn -> PCr
        // SPr-2 -> SPr
        let dest = self.load_operand(bus, src);
        if self.is_condition(condition) {
            let sp = Wrapping(self.sr.sp);
            bus.mem_write(self.mmu.to_physical((sp - Wrapping(1)).0), (self.sr.pc >> 8) as u8);
            bus.mem_write(self.mmu.to_physical((sp - Wrapping(2)).0), self.sr.pc as u8);
            self.sr.sp = (sp - Wrapping(2)).0;
            self.sr.pc = dest;
        }
    }

    pub(super) fn ret(&mut self, bus: &mut Bus, condition: Option<Condition>) {
        if self.is_condition(condition) {
            let sp = Wrapping(self.sr.sp);
            let lo = bus.mem_read(self.mmu.to_physical(self.sr.sp)) as u16;
            let hi = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0)) as u16;
            self.sr.sp = (sp + Wrapping(2)).0;
            self.sr.pc = lo | hi << 8;
        }
    }

    pub(super) fn halt(&mut self) {
        self.mode = Mode::Halt;
    }
}
