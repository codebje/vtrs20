mod board;
mod bus;
mod cpu;
mod types;

use bus::Bus;
use cpu::CPU;

fn main() {
    let mut cpu: CPU = CPU::new();
    let mut bus: Bus = Bus::new();

    // bus.add(peripheral);
    // ...

    //bus.reset();

    // while active {
    //     bus.cycle();
    // }
}
