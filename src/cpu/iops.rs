use crate::bus::Bus;
use crate::cpu::*;

/**
 * I/O Instructions
 *
 * Table 38 of Z8018x specification.
 */

impl CPU {
    // OUT0 (m), g
    pub(super) fn out0(&mut self, bus: &mut Bus, g: RegG) {
        let addr = bus.mem_read(self.mmu.to_physical(self.sr.pc + 2));
        let data = self.load_ghl(bus, g);
        bus.io_write(addr as u16, data);
        self.sr.pc += 3;
    }
}
