use bitstream_io::{BigEndian, BitReader};
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::io::FromRawFd;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::{io, thread};

use clap::{App, Arg, OsValues};

use emulator::bus::Bus;
use emulator::cpu::{Mode, Register, CPU};
use emulator::dma::*;
use emulator::prt::*;
use emulator::ram::*;

/*
  The information in a .REL file (Microsoft format) is in a bit stream:
- if the first bit is 0 then the next 8 bits will be loaded according to the
  value of the value of the location counter.  (To illustrate: 0-xxxx-xxxx).
- if the first bit is 1 then the next two bits will indicate the following:
     - 00 indicates a "special link item",
     - 01 indicates program relative; the next 16 bits will be loaded after
          being offset by the program segment origin
          (to illustrate: 0-01-xxxx-xxxx-xxxx-xxxx),
     - 10 indicates data relative; the next 16 bits will be loaded after
          being offset by the data segment origin
          (to illustrate: 0-10-xxxx-xxxx-xxxx-xxxx),
     - 11 indicates common relative; the next 16 bits will be loaded after
          being offset by the origin of the currently selected common block
          (to illustrate: 0-11-xxxx-xxxx-xxxx-xxxx).
The "special link item" consists of:
- 4 bit CONTROL field which selects one of 16 special link items
     - optional VALUE field consisting of a 2 bit address type field and a
       16 bit address field.  The address type field is as follows:
          - 00 indicates absolute address,
          - 01 indicates program relative address,
          - 10 indicates data relative address,
          - 11 indicates common relative address,
     - optional NAME field consisting of a 3 bit name byte count followed
       by the name in 8 bit ascii characters.
To illustrate, we have the bit stream as follows:
  CONTROL field followed by a VALUE filed and a NAME field
          1-00-xxxx-yy-zzzzZZZZzzzzZZZZ-XXX-<ascii characters>,
  CONTROL field followed by a NAME field only
          1-00-xxxx-XXX-<ascii characters>,
  CONTROL field followed by a VALUE field only
          1-00-xxxx-yy-zzzzZZZZzzzzZZZZ.
The following special link items are followed by the NAME field only:
     - 0000 entry symbol; the symbol indicated in the name field is defined
       in this module, so module should be linked if the current file is
       being searched,
     - 0001 select common block; the linker will use the location counter
       associated with the common block indicated in the name field for
       subsequent common relative items,
     - 0010 program name; the name of the relocatable module,
     - 0011 - not used,
     - 0100 - not used,
The following special link items are followed by VALUE and NAME fields:
     - 0101 define common size; the value field determines the amount of
       memory to be reserved for the common block described in the name field,
     - 0110 chain external; the value field contains the head of chain
       which ends with 0.  Each element of the chain is to be replaced with
       the value of the external symbol described in the name field,
     - 0111 define entry point; the value of the symbol in the name field
       is defined by the value field,
     - 1000 - not used.
The following special link items are followed by a VALUE field only:
     - 1001 external plus offset; the following 2 bytes in the current
       segment must be offset by the value of the value field after all
       chains have been processed,
     - 1010 define data size; the value field contains number of bytes in
       the data segment of the current module,
     - 1011 set location counter to the value determined by the value field,
     - 1100 chain address; the value field contains the head of a chain
       which ends with 0.  Each element of the chain is to be replaced with
       the current value of the location counter,
     - 1101 define program size; the value field contains the number of
       bytes in the program segment of the current module,
     - 1110 end of the current module; if the value field contains a value
       other than 0 then it is to be used as the start address for the
       program to be linked.  The next item in the file will start at the
       next byte boundary (the unused bits are "padded" with 0s).
The following special link item has no VALUE field or NAME field:
     - 1111 end of the file; follows the end module item of the last module
       in the file.
 */

#[derive(Debug)]
enum AddressType {
    Absolute,
    ProgramRelative,
    DataRelative,
    CommonRelative,
}

#[derive(Debug)]
enum Operator {
    StoreByte,
    StoreWord,
    HighByte,
    LowByte,
    Complement,
    Negate,
    Subtract,
    Add,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Debug)]
enum RelEntry {
    Absolute(u8),
    Relative(AddressType, u16),
    EntrySymbol(String),
    SelectCommon(String),
    ProgramName(String),
    Operand(AddressType, u16),
    ExternalOperand(String),
    Operation(Operator),
    CommonSize(AddressType, u16, String),
    ChainExternal(AddressType, u16, String),
    EntryPoint(AddressType, u16, String),
    ExtPlusOffset(AddressType, u16),
    DataSize(AddressType, u16),
    SetLocation(AddressType, u16),
    ChainAddress(AddressType, u16),
    TextSize(AddressType, u16),
    EndModule(AddressType, u16),
    EndFile(),
}

fn load_rel_string<R: Read>(reader: &mut BitReader<R, BigEndian>) -> Result<String, std::io::Error> {
    let len: u8 = reader.read(3)?;
    let mut buf = Vec::with_capacity(len as usize);
    buf.resize(len as usize, 0);
    reader.read_bytes(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

fn load_rel_type<R: Read>(reader: &mut BitReader<R, BigEndian>) -> Result<AddressType, std::io::Error> {
    let rel: u8 = reader.read(2)?;
    match rel {
        0b00 => Ok(AddressType::Absolute),
        0b01 => Ok(AddressType::ProgramRelative),
        0b10 => Ok(AddressType::DataRelative),
        0b11 => Ok(AddressType::CommonRelative),
        _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad relative bits")),
    }
}

fn load_rel_word<R: Read>(reader: &mut BitReader<R, BigEndian>) -> Result<u16, std::io::Error> {
    Ok(u16::from_be(reader.read(16)?))
}

fn rel_op(op: Option<&str>) -> Result<Operator, std::io::Error> {
    match op {
        Some("\x01") => Ok(Operator::StoreByte),
        Some("\x02") => Ok(Operator::StoreWord),
        Some("\x03") => Ok(Operator::HighByte),
        Some("\x04") => Ok(Operator::LowByte),
        Some("\x05") => Ok(Operator::Complement),
        Some("\x06") => Ok(Operator::Negate),
        Some("\x07") => Ok(Operator::Subtract),
        Some("\x08") => Ok(Operator::Add),
        Some("\x09") => Ok(Operator::Multiply),
        Some("\x0a") => Ok(Operator::Divide),
        Some("\x0b") => Ok(Operator::Modulo),
        Some(o) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("invalid operator {}", o),
        )),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "missing extension operator",
        )),
    }
}

fn load_rel_entry<R: Read>(reader: &mut BitReader<R, BigEndian>) -> Result<RelEntry, std::io::Error> {
    if reader.read_bit()? {
        let rel: u8 = reader.read(2)?;
        match rel {
            0b01 => Ok(RelEntry::Relative(AddressType::ProgramRelative, load_rel_word(reader)?)),
            0b10 => Ok(RelEntry::Relative(AddressType::DataRelative, load_rel_word(reader)?)),
            0b11 => Ok(RelEntry::Relative(AddressType::CommonRelative, load_rel_word(reader)?)),
            _ => {
                let control: u8 = reader.read(4)?;
                match control {
                    0b0000 => Ok(RelEntry::EntrySymbol(load_rel_string(reader)?)),
                    0b0001 => Ok(RelEntry::SelectCommon(load_rel_string(reader)?)),
                    0b0010 => Ok(RelEntry::ProgramName(load_rel_string(reader)?)),
                    0b0100 => {
                        let s = load_rel_string(reader)?;
                        match s.get(..1) {
                            Some("A") => Ok(RelEntry::Operation(rel_op(s.get(1..))?)),
                            Some("B") => Ok(RelEntry::ExternalOperand(s.get(1..).unwrap_or("").to_string())),
                            Some("C") => match s.as_bytes() {
                                [_, 0, vlo, vhi] => {
                                    Ok(RelEntry::Operand(AddressType::Absolute, (*vhi as u16) << 8 | (*vlo as u16)))
                                }
                                [_, 1, vlo, vhi] => Ok(RelEntry::Operand(
                                    AddressType::ProgramRelative,
                                    (*vhi as u16) << 8 | (*vlo as u16),
                                )),
                                _ => Err(std::io::Error::new(
                                    std::io::ErrorKind::InvalidData,
                                    format!("invalid operand extension {}", s),
                                )),
                            },
                            Some(t) => Err(std::io::Error::new(
                                std::io::ErrorKind::InvalidData,
                                format!("unknown extension {}", t),
                            )),
                            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid null extension")),
                        }
                    }
                    0b0101 => Ok(RelEntry::CommonSize(
                        load_rel_type(reader)?,
                        load_rel_word(reader)?,
                        load_rel_string(reader)?,
                    )),
                    0b0110 => Ok(RelEntry::ChainExternal(
                        load_rel_type(reader)?,
                        load_rel_word(reader)?,
                        load_rel_string(reader)?,
                    )),
                    0b0111 => Ok(RelEntry::EntryPoint(
                        load_rel_type(reader)?,
                        load_rel_word(reader)?,
                        load_rel_string(reader)?,
                    )),
                    0b1001 => Ok(RelEntry::ExtPlusOffset(load_rel_type(reader)?, load_rel_word(reader)?)),
                    0b1010 => Ok(RelEntry::DataSize(load_rel_type(reader)?, load_rel_word(reader)?)),
                    0b1011 => Ok(RelEntry::SetLocation(load_rel_type(reader)?, load_rel_word(reader)?)),
                    0b1100 => Ok(RelEntry::ChainAddress(load_rel_type(reader)?, load_rel_word(reader)?)),
                    0b1101 => Ok(RelEntry::TextSize(load_rel_type(reader)?, load_rel_word(reader)?)),
                    0b1110 => {
                        let val = Ok(RelEntry::EndModule(load_rel_type(reader)?, load_rel_word(reader)?));
                        reader.byte_align();
                        val
                    }
                    0b1111 => Ok(RelEntry::EndFile()),
                    _ => Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("bad control byte {:04b}", control),
                    )),
                }
            }
        }
    } else {
        Ok(RelEntry::Absolute(reader.read(8)?))
    }
}

fn io_err(s: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, s)
}

/**
 * Loads a REL file into RAM, relocating code to base <addr>.
 *
 * Does not support a data segment.
 */
fn load_rel(ram: &RAM, addr: u32, src: &str) -> Result<(), std::io::Error> {
    let f = File::open(src)?;
    let mut reader = BitReader::endian(f, BigEndian);

    let mut location = addr;

    let mut operands = Vec::new();
    let mut stores = Vec::new();

    /*
     * Extension("C\u{1}\u{0}\u{0}")        base+offset operand: base=addr, offset=0
     * Extension("C\u{0}\u{0}\u{1}")        base+offset operand: base=0, offset=256
     * Extension("A\n")                     operation: DIV
     * Extension("C\u{0}\u{1}\u{0}")        base+offset operand: base=0, offset=1
     * Extension("A\u{7}")                  operation: MINUS
     * Extension("A\u{1}")                  operation: BYTE
     */

    loop {
        let x = load_rel_entry(&mut reader)?;
        match x {
            RelEntry::ProgramName(name) => println!("Loading {} from {}...", name, src),
            RelEntry::EntrySymbol(name) => println!("  Entry symbol is {}", name),
            RelEntry::DataSize(_, 0) => (), // data is unsupported: a zero size is okay, anything else is not
            RelEntry::TextSize(_, sz) => println!("  Text size is {}", sz),
            RelEntry::Absolute(b) => {
                ram.write(location, &[b]);
                location = location + 1;
            }
            RelEntry::Relative(AddressType::ProgramRelative, w) => {
                let val = w + addr as u16;
                ram.write(location, &val.to_le_bytes());
                location = location + 2;
            }
            RelEntry::SetLocation(AddressType::ProgramRelative, l) => {
                location = l as u32 + addr;
            }
            RelEntry::EntryPoint(_, _, _) => (),
            RelEntry::EndModule(_, _) => (),
            RelEntry::EndFile() => break,
            RelEntry::Operand(AddressType::Absolute, v) => operands.push(v),
            RelEntry::Operand(AddressType::ProgramRelative, v) => operands.push(v + addr as u16),
            RelEntry::Operation(Operator::StoreByte) => {
                let o = operands
                    .pop()
                    .ok_or(io_err("Attempt to store byte with no operands on the stack"))?;
                stores.push((location, o as u8));
            }
            //RelEntry::Operation(Operator::StoreWord) => {
            //RelEntry::Operation(Operator::HighByte) => {
            //RelEntry::Operation(Operator::LowByte) => {
            //RelEntry::Operation(Operator::Complement) => {
            //RelEntry::Operation(Operator::Negate) => {
            RelEntry::Operation(Operator::Subtract) => {
                let o2 = operands
                    .pop()
                    .ok_or(io_err("Attempt to subtract with no operands on the stack"))?;
                let o1 = operands
                    .pop()
                    .ok_or(io_err("Attempt to subtract with only one operand on the stack"))?;
                operands.push(o1 - o2);
            }
            //RelEntry::Operation(Operator::Add) => {
            //RelEntry::Operation(Operator::Multiply) => {
            RelEntry::Operation(Operator::Divide) => {
                let o2 = operands
                    .pop()
                    .ok_or(io_err("Attempt to divide with no operands on the stack"))?;
                let o1 = operands
                    .pop()
                    .ok_or(io_err("Attempt to divide with only one operand on the stack"))?;
                operands.push(o1 / o2);
            }
            //RelEntry::Operation(Operator::Modulo) => {
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unsupported item {:?}", x),
                ))
            }
        }
    }

    // byte stores must happen after other decoding - the bitstream doesn't expect location to
    // increment in an extension, so the extension will be followed by an absolute byte write
    for (address, byte) in stores {
        ram.write(address, &[byte]);
    }

    Ok(())
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn dump_mem(ram: &RAM, addr: u32) {
    for row in 0..15 {
        let mut data: [u8; 16] = [0; 16];
        ram.read(addr + (row * 16), &mut data);
        print!("{:05x}  ", addr + (row * 16));
        for byte in 0..8 {
            print!("{:02x} ", data[byte]);
        }
        print!(" ");
        for byte in 8..16 {
            print!("{:02x} ", data[byte]);
        }
        println!("");
    }
}

/**
 * write_files
 *
 * The RAM disk begins at 0x20000 and spans 384k to the end of RAM. It has an allocation unit of
 * 2048 bytes per block, with two blocks reserved at the start of the disk for directory entries.
 * This allows up to 128 extents, with each extents reserving up to 32K of disk space for a file.
 *
 * The disk writer will allocate extents and blocks sequentially.
 */
fn write_files(ram: &RAM, files: OsValues) -> Result<(), std::io::Error> {
    let mut extent = 0;
    let mut block = 2;

    let cpm_valid = |c: &u8| (*c as char).is_ascii_alphanumeric();

    for file in files {
        let metadata = std::fs::metadata(file)?;
        if metadata.is_dir() {
            // TODO: add the whole directory
            eprintln!("warning: skipping directory {}", file.to_string_lossy());
        } else {
            let size = metadata.len();
            if block + (size + 2047) / 2048 > 191 {
                return Err(io_err(&format!(
                    "file {} is larger than remaining disk space ({}Kb)",
                    file.to_string_lossy(),
                    384 - block
                )));
            }
            if extent + (size + 32767) / 32768 > 128 {
                return Err(io_err(&format!(
                    "file {} requires {} extents, only {} remain",
                    file.to_string_lossy(),
                    (size + 15) / 16,
                    128 - extent
                )));
            }

            let path = Path::new(file);
            let stem = path
                .file_stem()
                .unwrap_or(OsStr::new("INVALID FILENAME"))
                .as_bytes()
                .to_ascii_uppercase();
            if stem.len() > 8 || !stem.iter().all(cpm_valid) {
                return Err(io_err(&format!(
                    "file {} must conform to the 8.3 filename limits of CP/M and be alphanumeric only",
                    file.to_string_lossy()
                )));
            }
            let ext = path.extension().unwrap_or(OsStr::new("")).as_bytes().to_ascii_uppercase();
            if ext.len() > 3 || !stem.iter().all(cpm_valid) {
                return Err(io_err(&format!(
                    "file {} must conform to the 8.3 filename limits of CP/M and be alphanumeric only",
                    file.to_string_lossy()
                )));
            }

            // write all needed extents
            for e in 0..(size + 32767) / 32768 {
                let size_here = size - e * 32768;
                let last_e = if size_here > 16383 { e * 2 + 1 } else { e * 2 };
                let data: [u8; 32] = [
                    0,
                    *stem.get(0).unwrap_or(&32),
                    *stem.get(1).unwrap_or(&32),
                    *stem.get(2).unwrap_or(&32),
                    *stem.get(3).unwrap_or(&32),
                    *stem.get(4).unwrap_or(&32),
                    *stem.get(5).unwrap_or(&32),
                    *stem.get(6).unwrap_or(&32),
                    *stem.get(7).unwrap_or(&32),
                    *ext.get(0).unwrap_or(&32),
                    *ext.get(1).unwrap_or(&32),
                    *ext.get(2).unwrap_or(&32),
                    last_e as u8,                           // extent# low
                    0,                                      // Bc, unused
                    0,                                      // extent# high
                    ((size - e * 32768 + 127) / 128) as u8, // # sectors in last block
                    block as u8,
                    if size > e * 32768 + 2048 * 1 { (block + 1) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 2 { (block + 2) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 3 { (block + 3) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 4 { (block + 4) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 5 { (block + 5) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 6 { (block + 6) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 7 { (block + 7) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 8 { (block + 8) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 9 { (block + 9) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 10 { (block + 10) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 11 { (block + 11) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 12 { (block + 12) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 13 { (block + 13) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 14 { (block + 14) as u8 } else { 0 },
                    if size > e * 32768 + 2048 * 15 { (block + 15) as u8 } else { 0 },
                ];
                ram.write(0x20000 + (extent * 32) as u32, &data);
                extent = extent + 1;
            }

            ram.load_file(0x20000 + (block * 2048) as u32, file)?;
            block = block + (size + 2047) / 2048;
        }
    }

    Ok(())
}

fn spawn_input() -> Receiver<char> {
    let (tx, rx) = mpsc::channel::<char>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer).unwrap();
        for ch in buffer.chars() {
            tx.send(ch).unwrap();
        }
    });
    rx
}

fn main() -> Result<(), std::io::Error> {
    let matches = App::new("Virtual TRS-20 - Run CP/M")
        .version("1.0")
        .about("Run CP/M")
        .arg(
            Arg::with_name("COM")
                .short("c")
                .long("com")
                .value_name("COM file")
                .help("Run a transient command directly")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("files")
                .multiple(true)
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Add a file to the RAM disk")
                .takes_value(true),
        )
        .get_matches();

    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let ram = Rc::new(RAM::new(0x00000, 0x80000));

    // load ZSDOS
    load_rel(&ram, 0xe000, "zcpr.rel")?;
    load_rel(&ram, 0xe800, "zsdos.rel")?;

    // Fill in a BIOS
    ram.write(
        0xf600,
        &[
            0, 0, 0, // cold boot: 0xf600 - continues into warm boot
            0xc3, 0x00, 0xe0, // warm boot: 0xf603 - jumps to zcpr1
            0xc9, 0, 0, // console status: 0xf606
            0xc9, 0, 0, // console in: 0xf609
            0xc9, 0, 0, // console out: 0xf60c
            0xc9, 0, 0, // list out: 0xf60f
            0xc9, 0, 0, // punch out: 0xf612
            0xc9, 0, 0, // reader in: 0xf615,
            0xc9, 0, 0, // home disk: 0xf618,
            0xc9, 0, 0, // select disk: 0xf61b,
            0xc9, 0, 0, // select track: 0xf61e,
            0xc9, 0, 0, // select sector: 0xf621,
            0xc9, 0, 0, // set dma address: 0xf624,
            0xc9, 0, 0, // read 128 bytes: 0xf627,
            0xc9, 0, 0, // write 128 bytes: 0xf62a,
            0xc9, 0, 0, // list status: 0xf62d,
            0xc9, 0, 0, // sector translate: 0xf630
        ],
    );

    // disk parameters at 0xfa00
    ram.write(
        0xfa00,
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0x00, 0xfb, 0x10, 0xfa, 0, 0, 0x80, 0xfb, // DPH
            16, 0, 4, 15, 1, 191, 0, 127, 0, 0b11000000, 0, 0, 0, 0, 0, // DPB
        ],
    );

    // erase the RAM disk
    for base in 0x20000..0x7FFFF {
        ram.write(base, &[0xe5]);
    }

    // write files into the RAM disk
    for files in matches.values_of_os("files") {
        write_files(&ram, files)?;
    }

    bus.add(ram.clone());

    let prt = Rc::new(PRT::new());
    bus.add(prt);

    let dma = Rc::new(DMA::new());
    bus.add(dma);

    cpu.reset();

    match matches.value_of("COM") {
        Some(com) => {
            ram.load_file(0x100, com)?;
            cpu.write_reg(Register::PC, 0x100);
            ram.write(0, &[0x76, 0x03, 0xf6, 0x00, 0x00, 0xc3, 0x06, 0xe8]);
        }
        None => cpu.write_reg(Register::PC, 0xf600),
    }

    //let mut stdout = stdout();
    //use std::os::unix::io::FromRawFd;
    let mut stdout = unsafe { File::from_raw_fd(1) };

    let input_ch = spawn_input();
    let mut input = None;

    let mut track = 0;
    let mut sector = 0;
    let mut dma = 0;

    loop {
        let pc = cpu.reg(Register::PC);
        match pc {
            0xf600 => {
                // cold boot
                // zeropage: jp warmboot, iobyte=0, disk=0, jp bdos
                write!(stdout, "vTRS-20 CP/M online\r\n")?;
            }
            0xf603 => {
                // warm boot
                ram.write(0, &[0xc3, 0x03, 0xf6, 0x00, 0x00, 0xc3, 0x06, 0xe8]);
                cpu.write_reg(Register::C, 0);
                cpu.write_reg(Register::PC, 0xe101);
            }
            0xf606 => {
                // console status
                if input.is_none() {
                    input = input_ch.try_recv().ok();
                }
                cpu.write_reg(Register::A, if input.is_none() { 0x00 } else { 0xff });
            }
            0xf609 => {
                // console in
                if input.is_none() {
                    input = Some(input_ch.recv().map_err(|_| io_err("input channel died"))?);
                }
                cpu.write_reg(Register::A, input.unwrap_or('\0') as u16);
                input = None;
            }
            0xf60c => {
                // console out
                write!(stdout, "{}", (cpu.reg(Register::C) as u8 & 0x7f) as char)?;
            }
            0xf60f => {} // list out
            0xf612 => {} // punch out
            0xf615 => {
                // reader in
                cpu.write_reg(Register::A, 0x1a);
            }
            0xf618 => {
                // home disk
                track = 0;
            }
            0xf61b => {
                // select disk
                let disk = cpu.reg(Register::C);
                cpu.write_reg(Register::HL, if disk == 0 { 0xfa00 } else { 0 });
            }
            0xf61e => {
                // select track
                track = cpu.reg(Register::BC);
            }
            0xf621 => {
                // select sector
                sector = cpu.reg(Register::BC);
            }
            0xf624 => {
                // set dma address
                dma = cpu.reg(Register::BC);
            }
            0xf627 => {
                // read 128 bytes
                let offset = ((track as u32) * 16 + (sector as u32)) * 128 + 0x20000;
                let mut buf = Vec::with_capacity(128);
                buf.resize(128, 0);
                ram.read(offset, &mut buf);
                ram.write(dma as u32, &buf);
                cpu.write_reg(Register::A, 0);
            }
            0xf62a => {
                // write 128 bytes
                let offset = ((track as u32) * 16 + (sector as u32)) * 128 + 0x20000;
                let mut buf = Vec::with_capacity(128);
                buf.resize(128, 0);
                ram.read(dma as u32, &mut buf);
                ram.write(offset, &buf);
                cpu.write_reg(Register::A, 0);
            }
            0xf62d => {
                // list status
                cpu.write_reg(Register::A, 0);
            }
            0xf630 => {
                // sector translate
                cpu.write_reg(Register::HL, cpu.reg(Register::BC));
            }
            _ => (),
        }
        if pc >= 0x100 && pc <= 0x700 {
            print_cpu(&mut cpu, &mut bus);
        }
        if cpu.mode != Mode::OpCodeFetch {
            break;
        }
        cpu.cycle(&mut bus);
    }
    dump_mem(&ram, 0x7d0);
    dump_mem(&ram, 0xe3e0);

    Ok(())
}
