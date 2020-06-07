use crate::cpu::CPU;

/**
 * Special Control Instructions
 *
 * Table 47 of Z8018x specification.
 */

impl CPU {
    pub(super) fn nop(&mut self) {
        self.sr.pc += 1;
    }
}
