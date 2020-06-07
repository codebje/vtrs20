use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * Program Control Instructions
 *
 * Table 45 of Z8018x specification.
 */

impl CPU {
    // JP mn
    pub(super) fn jp(&mut self, bus: &mut Bus) {
        let n = bus.mem_read(self.mmu.to_physical(self.sr.pc + 1));
        let m = bus.mem_read(self.mmu.to_physical(self.sr.pc + 2));
        let addr = n as u16 | (m as u16) << 8;
        println!("jump to {}", addr);
        self.sr.pc = addr;
    }
}
