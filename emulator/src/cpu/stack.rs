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
        let lo = bus.mem_read(self.mmu.to_physical(self.sr.sp), false) as u16;
        let hi = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0), false) as u16;
        self.store_operand(bus, dst, lo | hi << 8);
        self.sr.sp = (sp + Wrapping(2)).0;
    }

    // Exchanges
    pub(super) fn exchange(&mut self, bus: &mut Bus, exchange: Exchange) {
        match exchange {
            Exchange::AF_AFs => {
                std::mem::swap(&mut self.gr.a, &mut self.gr_.a);
                std::mem::swap(&mut self.gr.f, &mut self.gr_.f);
            }
            Exchange::DE_HL => std::mem::swap(&mut self.gr.hl, &mut self.gr.de),
            Exchange::X => {
                std::mem::swap(&mut self.gr.bc, &mut self.gr_.bc);
                std::mem::swap(&mut self.gr.de, &mut self.gr_.de);
                std::mem::swap(&mut self.gr.hl, &mut self.gr_.hl);
            }
            Exchange::SP_HL => {
                let sp = Wrapping(self.sr.sp);
                let lo = bus.mem_read(self.mmu.to_physical(self.sr.sp), false) as u16;
                let hi = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0), false) as u16;
                bus.mem_write(self.mmu.to_physical(self.sr.sp), self.gr.hl as u8);
                bus.mem_write(self.mmu.to_physical((sp + Wrapping(1)).0), (self.gr.hl >> 8) as u8);
                self.gr.hl = lo | hi << 8;
            }
            Exchange::SP_IX => {
                let sp = Wrapping(self.sr.sp);
                let lo = bus.mem_read(self.mmu.to_physical(self.sr.sp), false) as u16;
                let hi = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0), false) as u16;
                bus.mem_write(self.mmu.to_physical(self.sr.sp), self.sr.ix as u8);
                bus.mem_write(self.mmu.to_physical((sp + Wrapping(1)).0), (self.sr.ix >> 8) as u8);
                self.sr.ix = lo | hi << 8;
            }
            Exchange::SP_IY => {
                let sp = Wrapping(self.sr.sp);
                let lo = bus.mem_read(self.mmu.to_physical(self.sr.sp), false) as u16;
                let hi = bus.mem_read(self.mmu.to_physical((sp + Wrapping(1)).0), false) as u16;
                bus.mem_write(self.mmu.to_physical(self.sr.sp), self.sr.iy as u8);
                bus.mem_write(self.mmu.to_physical((sp + Wrapping(1)).0), (self.sr.iy >> 8) as u8);
                self.sr.iy = lo | hi << 8;
            }
        }
    }
}
