use crate::bus::Bus;
use crate::bus::Signals;
use crate::types::Signal;

mod mmu;

enum Mode {
    Reset,
    OpCodeFetch,
    // Operand1Fetch(u8),
    // Operand2Fetch(u8, u8),
    //Refresh,
    //IntAckNMI,
    //IntAckINT0,
    //IntAckOther,
    //BusRelease,
    //Halt,
    //Sleep,
    //DMARead,
    //DMAWrite,
    //IORead,
    //IOWrite,
}

pub struct CPU {
    // internal state
    mode: Mode,
    mmu: mmu::MMU,
    pc: u16,
    dcntl: u8,
}

impl CPU {
    // Create a new CPU. The CPU will be in Reset by default.
    pub fn new() -> CPU {
        CPU {
            mode: Mode::Reset,
            mmu: mmu::MMU::new(),
            pc: 0x0000,
            dcntl: 0b1111_0000,
        }
    }

    // reset the CPU.
    pub fn reset<'b>(&mut self, bus: &'b mut Bus) {
        self.mode = Mode::Reset;
        self.dcntl = 0b1111_0000;
        self.pc = 0x0000;
        self.mmu.reset();
        bus.signals.raise(Signal::M1);
        bus.signals.raise(Signal::ST);
        bus.signals.raise(Signal::MREQ);
        bus.signals.raise(Signal::RD);
    }

    // Run one machine cycle. This will assert various signals on the bus to do its job.
    pub fn cycle(&mut self, bus: &mut Bus) {
        match self.mode {
            Mode::Reset => (),
            Mode::OpCodeFetch => self.fetch_opcode(bus),
            //Mode::OpCodeFetch(opcode) => self.decode(opcode),
        }
    }

    // Pull an opcode from the bus
    fn fetch_opcode(&mut self, bus: &mut Bus) {
        bus.signals.lower(Signal::M1);
        bus.signals.lower(Signal::ST);
        bus.signals.lower(Signal::MREQ);
        bus.signals.lower(Signal::RD);

        let _data = bus.read(self.mmu.to_physical(self.pc));

        bus.signals.raise(Signal::M1);
        bus.signals.raise(Signal::ST);
        bus.signals.raise(Signal::RD);
        bus.signals.raise(Signal::MREQ);

        // decode the instruction

        self.mode = Mode::Reset;
    }

    // Decode an opcode.
    //fn decode(&mut self, opcode: u8) {}
}
