use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * 16-bit Load
 *
 * Table 42 of Z8018x specification.
 */

impl CPU {
    // Execute LD ww, mn
    pub(super) fn ld_ww_mn(&mut self, bus: &mut Bus, ww: u8) {
        let n = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1)) as u16;
        let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 2)) as u16;
        let mn = m << 8 | n;

        match ww {
            0b00 => self.gr.bc = mn,
            0b01 => self.gr.de = mn,
            0b10 => self.gr.hl = mn,
            0b11 => self.sr.sp = mn,
            _ => self.error(),
        }

        self.sr.pc += 3;
    }
}
