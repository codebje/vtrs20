use crate::bus::Bus;
use crate::cpu::*;

/**
 * 16-bit Load
 *
 * Table 42 of Z8018x specification.
 */

impl CPU {
    // Execute LD ww, mn
    pub(super) fn ld_ww_mn(&mut self, bus: &mut Bus, ww: RegW) {
        let n = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
        let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 2)) as u16;
        let mn = m << 8 | n;

        match ww {
            RegW::BC => self.gr.bc = mn,
            RegW::DE => self.gr.de = mn,
            RegW::HL => self.gr.hl = mn,
            RegW::SP => self.sr.sp = mn,
        }

        self.sr.pc += 3;
    }
}
