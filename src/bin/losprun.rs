use std::rc::Rc;

use clap::{App, Arg};

use emulator::asci::*;
use emulator::bus::Bus;
use emulator::cpu::{Mode, Register, CPU};
use emulator::dma::*;
use emulator::prt::*;
use emulator::ram::*;

fn print_cpu(cpu: &mut CPU, bus: &mut Bus) {
    let mut opcodes: [u8; 4] = [0, 0, 0, 0];
    cpu.get_current_opcodes(bus, &mut opcodes);
    let flags = cpu.reg(Register::F);
    println!(
        "PC=${:04x}, SP=${:04x} \
                A=${:02x} BC=${:04x} DE=${:04x} HL=${:04x} IX=${:04x} IY=${:04x} \
                {}{}-{}-{}{}{}    {}",
        cpu.reg(Register::PC),
        cpu.reg(Register::SP),
        cpu.reg(Register::A),
        cpu.reg(Register::BC),
        cpu.reg(Register::DE),
        cpu.reg(Register::HL),
        cpu.reg(Register::IX),
        cpu.reg(Register::IY),
        if flags & 0b1000_0000 != 0 { 'S' } else { 's' },
        if flags & 0b0100_0000 != 0 { 'Z' } else { 'z' },
        if flags & 0b0001_0000 != 0 { 'H' } else { 'h' },
        if flags & 0b0000_0100 != 0 { 'P' } else { 'p' },
        if flags & 0b0000_0010 != 0 { 'N' } else { 'n' },
        if flags & 0b0000_0001 != 0 { 'C' } else { 'c' },
        emulator::disasm::disasm(&opcodes),
    );
}

fn print_ident(cpu: &mut CPU, bus: &mut Bus) {
    let mut addr = cpu.reg(Register::HL) as u32;
    loop {
        let byte = bus.mem_read(addr, false);
        if byte == 0 {
            break;
        }
        print!("{}", byte as char);
        addr = addr + 1;
    }
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Virtual TRS-20 - Losp Runner")
        .version("1.0")
        .about("Run the Losp interpreter")
        .arg(Arg::with_name("BIN").required(true).index(1))
        .get_matches();

    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let ram = Rc::new(RAM::new(0x00000, 0x80000));
    ram.load_file(0x100, matches.value_of("BIN").unwrap())?;

    bus.add(ram.clone());

    let prt = Rc::new(PRT::new());
    bus.add(prt);

    let dma = Rc::new(DMA::new());
    bus.add(dma);

    match matches.value_of("tty") {
        Some(tty) => {
            let uart = ASCI::new(Channel::CH0, tty);
            bus.add(Rc::new(uart));
        }
        None => (),
    }

    cpu.reset();

    let mut input = "(ident \"a\tstring\" $id2* #f #\\#) ... , ,foo ,@ ;45 \n\
                    comment-ok 32767 -32768 #d33 #x4f #b0011_1100"
        .chars();

    loop {
        let pc = cpu.reg(Register::PC);
        if pc == 0x10e {
            cpu.write_reg(Register::A, input.next().unwrap_or('\x1a') as u16);
        }
        if pc == 0x10f {
            let tok = cpu.reg(Register::A);
            match tok {
                1 => {
                    print!("IDENT=");
                    print_ident(&mut cpu, &mut bus)
                }
                2 => print!("TRUE"),
                3 => print!("FALSE"),
                4 => print!("NUM=??"),
                5 => print!("CHAR={}", (cpu.reg(Register::L) as u8) as char),
                6 => {
                    print!("STRING=\"");
                    print_ident(&mut cpu, &mut bus);
                    print!("\"");
                }
                7 => print!("("),
                8 => print!(")"),
                9 => print!("#("),
                10 => print!("'"),
                11 => print!("`"),
                12 => print!(","),
                13 => print!(",@"),
                14 => print!("."),
                15 => print!("EOF"),
                _ => print!("TOK={}", tok),
            }
            print!(" ");
        }
        if cpu.mode != Mode::OpCodeFetch {
            break;
        }
        cpu.cycle(&mut bus);
    }
    println!("HALT");
    print_cpu(&mut cpu, &mut bus);

    Ok(())
}
