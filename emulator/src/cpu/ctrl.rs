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

    // JR j
    pub(super) fn jr(&mut self, bus: &mut Bus, src: Operand, condition: Option<Condition>) {
        let j = self.load_operand(bus, src) as i8;
        if self.is_condition(condition) {
            self.sr.pc = (Wrapping(self.sr.pc) + Wrapping(j as u16)).0;
        }
    }

    // djnz
    pub(super) fn djnz(&mut self, bus: &mut Bus) {
        let j = self.load_operand(bus, Operand::Immediate());
        let b = (Wrapping(self.reg(Register::B)) - Wrapping(1)).0;
        self.write_reg(Register::B, b);
        if b != 0 {
            self.sr.pc = self.sr.pc + j + 2;
        }
    }

    pub(super) fn call(&mut self, bus: &mut Bus, src: Operand, condition: Option<Condition>) {
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

    // this looks a hecking lot like it can just jump anywhere, which it can.
    // dispatch will invoke it only for specific vector addresses.
    pub(super) fn rst(&mut self, vec: u16) {
        self.sr.pc = vec;
    }

    pub(super) fn halt(&mut self) {
        self.mode = Mode::Halt;
    }
}
