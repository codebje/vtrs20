use crate::bus::Bus;
use crate::cpu::CPU;

/**
 * I/O Instructions
 *
 * Table 46 of Z8018x specification.
 */

impl CPU {
    // OUT0 (m), g
    pub(super) fn out0(&mut self, bus: &mut Bus, g: u8) {
        if g == 0b110 {
            // OUT0 (m), (HL) not defined
            self.error();
        } else {
            let addr = bus.mem_read(self.mmu.to_physical(self.sr.pc + 2));
            match self.load_g_hl(bus, g) {
                Ok(data) => {
                    bus.io_write(addr as u16, data);
                    self.sr.pc += 3;
                }
                Err(()) => (),
            }
        }
    }
}
