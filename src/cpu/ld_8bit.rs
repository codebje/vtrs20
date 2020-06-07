use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * 8-bit Load
 *
 * Table 41 of Z8018x specification.
 */

impl CPU {
    // Execute LD g, m or LD (HL), m
    pub(super) fn ld_g_m(&mut self, bus: &mut Bus, g: u8) {
        let imm = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1));
        println!("store {} to {}", imm, g);
        self.store_g_hl(bus, g, imm);
        self.sr.pc += 2;
    }
}
