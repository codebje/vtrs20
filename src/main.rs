mod board;
mod bus;
mod cpu;
mod ram;
mod rom;
mod types;

use std::rc::Rc;

use board::Board;
use bus::Bus;
use cpu::CPU;
use ram::RAM;
use rom::ROM;

fn basic_rom() -> Vec<u8> {
    let mut data = vec![
        0x3e, 0x00, 0xed, 0x39, 0x36, 0xc3, 0x38, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0xc3, 0x38, 0x00,
    ];
    data.resize(0x40000, 0u8);

    data
}

fn main() {
    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let mut board = Board::new(&mut cpu, &mut bus);

    // ROM must be added before RAM to allow it to override RAM at reset
    let rom = ROM::new(0x80000, basic_rom());
    board.add(Rc::new(rom));

    let ram = RAM::new(0x00000, 0x80000);
    board.add(Rc::new(ram));

    board.reset();
    board.cycle();
}
