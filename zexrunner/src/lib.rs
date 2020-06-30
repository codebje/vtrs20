use proc_macro2::{Literal, Punct, Spacing, Span};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;
use std::rc::Rc;
use std::ops::{Index, IndexMut};
use std::slice;
use std::mem;

use emulator::cpu::{Register, CPU};
use emulator::bus::{Bus, Peripheral};
use emulator::ram::RAM;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
#[repr(C)]
pub struct ZexState {
    pub instruction: u32,
    pub operand: u16,
    pub iy: u16,
    pub ix: u16,
    pub hl: u16,
    pub de: u16,
    pub bc: u16,
    pub f: u8,
    pub a: u8,
    pub sp: u16,
}

// Treat a ZexState as an indexable byte array
impl Index<usize> for ZexState {
    type Output = u8;
    
    // This remaps the ZexState pointer as a byte pointer. Note that
    // endianness is not considered.
    fn index(&self, i: usize) -> &Self::Output {
        let p: *const ZexState = self;
        let p: *const u8 = p as *const u8;
        let bytes: &[u8] = unsafe {
            slice::from_raw_parts(p, mem::size_of::<ZexState>())
        };
        &bytes[i]
    }
}

impl IndexMut<usize> for ZexState {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        let p: *mut ZexState = self;
        let p: *mut u8 = p as *mut u8;
        let bytes: &mut [u8] = unsafe {
            slice::from_raw_parts_mut(p, mem::size_of::<ZexState>())
        };
        &mut bytes[i]
    }    
}

#[test]
fn test_index_mut() {
    let mut z = ZexState {
        instruction: 0,
        operand: 0,
        iy: 0,
        ix: 0,
        hl: 0,
        de: 0,
        bc: 0,
        f: 0,
        a: 21,
        sp: 0,
    };

    // Because ZexState has a repr(C), but is not packed, some platforms
    // may introduce alignment padding. This test will always succeed on
    // x86 but may fail on other platforms.
    z[16] = 42;
    assert_eq!(z.f, 42);
    assert_eq!(z[17], 21);
    assert_eq!(mem::size_of::<ZexState>(), 20);
}

#[allow(dead_code)]
fn field_ref<U>(tokens: &mut proc_macro2::TokenStream, name: &str, value: U)
where
    U: Into<proc_macro2::TokenTree>,
{
    tokens.append(Ident::new(name, Span::call_site()));
    tokens.append(Punct::new(':', Spacing::Alone));
    tokens.append(value);
    tokens.append(Punct::new(',', Spacing::Alone));
}

impl ToTokens for ZexState {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut fields = proc_macro2::TokenStream::new();
        field_ref(&mut fields, "instruction", Literal::u32_suffixed(self.instruction));
        field_ref(&mut fields, "operand", Literal::u16_suffixed(self.operand));
        field_ref(&mut fields, "iy", Literal::u16_suffixed(self.iy));
        field_ref(&mut fields, "ix", Literal::u16_suffixed(self.ix));
        field_ref(&mut fields, "hl", Literal::u16_suffixed(self.hl));
        field_ref(&mut fields, "de", Literal::u16_suffixed(self.de));
        field_ref(&mut fields, "bc", Literal::u16_suffixed(self.bc));
        field_ref(&mut fields, "f", Literal::u8_suffixed(self.f));
        field_ref(&mut fields, "a", Literal::u8_suffixed(self.a));
        field_ref(&mut fields, "sp", Literal::u16_suffixed(self.sp));
        tokens.append_all(quote! {
            ZexState {
                #fields
            };
        });
    }
}

fn count_bits(state: &ZexState) -> u32 {
    state.instruction.count_ones()
        + state.operand.count_ones() as u32
        + state.iy.count_ones() as u32
        + state.ix.count_ones() as u32
        + state.hl.count_ones() as u32
        + state.de.count_ones() as u32
        + state.bc.count_ones() as u32
        + state.f.count_ones() as u32
        + state.a.count_ones() as u32
        + state.sp.count_ones() as u32
}

/*
; 0xb339    45881   0b1011_0011_0011_1001 
; 0xb338    45880   0b1011_0011_0011_1000 
; 0xf821    63521   0b1111_1000_0010_0001 

854 c=0x42 0b0100_0010  a=0x38 0b0011_1000  counter=01 00 00 00 00 ...
855                     a=     0b0001_1100  cf=0
859 c=     0b0100_0010  a=     0b0100_0010
860                     a=     0b0010_0001  cf=0
861 c=     0b0010_0001
862                     a=     0b0001_1100
864 b=7, loop to 855
855                     a=     0b0000_1110 cf=0
859                     a=     0b0010_0001
860                     a=     0b1001_0000 cf=1
861 c=     0b1001_0000
862                     a=     0b0000_1110
864 b=6, loop to 855
855                     a=     0b0000_0111 cf=0
858                     a=     0b0000_0000
859                     a=     0b1001_0000
860                     a=     0b0100_1000 cf=0
861 c=     0b0100_1000
862                     a=     0b0000_0111
864 b=5, loop to 855
855                     a=     0b1000_0011 cf=1
858                     a=     0b0000_0001
859                     a=     0b0100_1001
860                     a=     0b1010_0100 cf=1
861 c=     0b1010_0100
862                     a=     0b1000_0011
864 b=4, loop to 855 (a always zero at 859 from here on in)
855                     a=     0b1100_0001 cf=1
859                     a=     0b1010_0100
860                     a=     0b0101_0010 cf=0
862 c=0b0101_0010       a=     0b1100_0001
864 b=3, loop to 855
855                     a=     0b1110_0000 cf=1
859                     a=     0b0101_0010
862 c=     0b0010_1001  a=     0b1110_0000
864 b=2, loop to 855
855                     a=     0b0111_0000 cf=0
862 c=     0b1001_0100
864 b=1, loop to 855
855                     a=     0b0011_1000 cf=0
862 c=     0b0100_1010
864 b=0, done
888 output byte went from 0x42 to 0x4a - lowest bit of mask is set

854 c=0x42 0b0100_0010  a=0x38 0b0011_1000  counter=01 00 00 00 00 ...

871 c=0b0011_1001 a=0b1111_1111
-- loop, b=8
872 rrca        c=0b0011_1001 a=0b1111_1111 C
873 push af     c=0b0011_1001 a=0b1111_1111 C
874 ld   a,0    c=0b0011_1001 a=0b0000_0000 C
875 call...     c=0b0011_1001 a=0b0000_0001 Z
876 xor  c      c=0b0011_1001 a=0b0011_1000
877 rrca        c=0b0011_1001 a=0b0001_1100
878 ld   c,a    c=0b0001_1100 a=0b0001_1100
879 pop af      c=0b0001_1100 a=0b1111_1111
880 dec b
881 jp nz,..
-- loop, b=7 (873, 874, 875 always result in a=0)
872 rrca        c=0b0001_1100 a=0b1111_1111 C
876 xor  c      c=0b0001_1100 a=0b0001_1100
877 rrca        c=0b0001_1100 a=0b0000_1110
878 ld   c,a    c=0b0000_1110 a=0b0000_1110
879 pop af      c=0b0000_1110 a=0b1111_1111
-- loop, b=6
876 xor  c      c=0b0000_1110 a=0b0000_1110
878 ld   c,a    c=0b0000_0111
-- loop b=5
876 xor  c      a=0b0000_0111
878 ld   c,a    c=0b1000_0011
-- loop b=4
878 ld   c,a    c=0b1100_0001
-- loop b=3
878 ld   c,a    c=0b1110_0000
-- loop b=2
878 ld   c,a    c=0b0111_0000
-- loop b=1
878 ld   c,a    c=0b0011_1000



*/

fn flip_state(state: &mut ZexState, mask: &ZexState, count: &num::BigUint) {

    let mut v = count.clone();

    for i in 0..20 {
        for b in 0..8 {
            if mask[i] & (1 << b) != 0 {
                match v.trailing_zeros() {
                    Some(0) => state[i] ^= 1 << b,
                    _ => (),
                }
                v >>= 1;
            }
        }
    }

}

pub fn zex_run_test(init: &ZexState, incr: &ZexState, flip: &ZexState, mask: u8) -> u32 {

    // work out flip and increment loop sizes
    let incr_bits = count_bits(&incr);
    let flip_bits = count_bits(&flip);
    let flip_count = num::BigUint::from(2u8).pow(flip_bits);
    let incr_count = num::BigUint::from(2u8).pow(incr_bits);

    let mut shifter = num::BigUint::from(1u8);
    let mut first_test = true;  // the very first test does not get set up: this is a bug in zexdoc
    let mut crc = 0xffffffff;
    while shifter <= flip_count {
        let mut counter = num::BigUint::from(0u8);
        while counter < incr_count {
            let mut test_state = init.clone();
            if !first_test {
                flip_state(&mut test_state, &incr, &counter);
                flip_state(&mut test_state, &flip, &shifter);
            } else {
                first_test = false;
            }
            crc = zex_execute(&test_state, mask, crc);
            // return crc;
            counter = counter + 1u8;
        }
        shifter <<= 1;
    }
    
    crc
}

// Execute one instruction with a specific CPU state, returning an updated CRC
pub fn zex_execute(state: &ZexState, mask: u8, crc: u32) -> u32 {
    let mut bus = Bus::new();
    let mut cpu = CPU::new(&mut bus);
    let ram = Rc::new(RAM::new(0x0000, 0x10000));
    bus.add(ram.clone());

    ram.write(
        0x103,
        &[
            (state.operand >> 8) as u8,
            state.operand as u8,
            (state.iy >> 8) as u8,
            state.iy as u8,
            (state.ix >> 8) as u8,
            state.ix as u8,
            (state.hl >> 8) as u8,
            state.hl as u8,
            (state.de >> 8) as u8,
            state.de as u8,
            (state.bc >> 8) as u8,
            state.bc as u8,
            state.a,
            state.f,
            (state.sp >> 8) as u8,
            state.sp as u8,
            (state.instruction >> 24) as u8,
            (state.instruction >> 16) as u8,
            (state.instruction >> 8) as u8,
            (state.instruction >> 0) as u8,
        ]
    );

    // Set up the CPU
    cpu.reset();
    cpu.write_reg(Register::SP, state.sp);
    cpu.write_reg(Register::A, state.a as u16);
    cpu.write_reg(Register::F, state.f as u16);
    cpu.write_reg(Register::BC, state.bc);
    cpu.write_reg(Register::DE, state.de);
    cpu.write_reg(Register::HL, state.hl);
    cpu.write_reg(Register::IX, state.ix);
    cpu.write_reg(Register::IY, state.iy);
    cpu.write_reg(Register::PC, 0x113);

    // Run one instruction
    cpu.cycle(&mut bus);

    // Update the CRC
    let mut result = crc;
    result = updcrc(result, ram.mem_read(0x104).unwrap_or(0));
    result = updcrc(result, ram.mem_read(0x103).unwrap_or(0));
    result = updcrc(result, (cpu.reg(Register::IY) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::IY) >> 8) as u8);
    result = updcrc(result, (cpu.reg(Register::IX) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::IX) >> 8) as u8);
    result = updcrc(result, (cpu.reg(Register::HL) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::HL) >> 8) as u8);
    result = updcrc(result, (cpu.reg(Register::DE) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::DE) >> 8) as u8);
    result = updcrc(result, (cpu.reg(Register::BC) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::BC) >> 8) as u8);
    result = updcrc(result, cpu.reg(Register::F) as u8 & mask);
    result = updcrc(result, cpu.reg(Register::A) as u8);
    result = updcrc(result, (cpu.reg(Register::SP) >> 0) as u8);
    result = updcrc(result, (cpu.reg(Register::SP) >> 8) as u8);

    // println!("Test state: instr={:08x} hl={:04x}->{:04x} f={:08b} crc={:08x}",
    //     state.instruction, state.hl, cpu.reg(Register::HL), cpu.reg(Register::F), result);

    result
}

#[test]
fn exec_tests() {
    // test that an SBC HL,DE
    let state = ZexState {
        instruction: 0xed420000,
        operand: 0x2c83,
        iy: 0x4f88,
        ix: 0xf22b,
        hl: 0xb339,
        de: 0x7e1f,
        bc: 0x1563,
        f: 0xd3,
        a: 0x89,
        sp: 0x465e,
    };

    assert_eq!(zex_execute(&state, 0xff, 0xffffffff), 0x96e7a894, "CRC is correct");
}

fn updcrc(crcval: u32, byte: u8) -> u32 {
    let idx = (crcval as u8 ^ byte) as usize;

    (crcval >> 8) ^ CRCTAB[idx]
}

#[test]
fn crctest() {
    assert_eq!(updcrc(0xffffffff, 0x2c), 0x1f257c91);
    assert_eq!(updcrc(0x1f257c91, 0x83), 0xf3a65434);
    assert_eq!(updcrc(0xf3a65434, 0x88), 0xc22459f3);
}

static CRCTAB: [u32; 256] = [
    0x00000000, 0x77073096, 0xee0e612c, 0x990951ba, 0x076dc419, 0x706af48f, 0xe963a535, 0x9e6495a3, 0x0edb8832,
    0x79dcb8a4, 0xe0d5e91e, 0x97d2d988, 0x09b64c2b, 0x7eb17cbd, 0xe7b82d07, 0x90bf1d91, 0x1db71064, 0x6ab020f2,
    0xf3b97148, 0x84be41de, 0x1adad47d, 0x6ddde4eb, 0xf4d4b551, 0x83d385c7, 0x136c9856, 0x646ba8c0, 0xfd62f97a,
    0x8a65c9ec, 0x14015c4f, 0x63066cd9, 0xfa0f3d63, 0x8d080df5, 0x3b6e20c8, 0x4c69105e, 0xd56041e4, 0xa2677172,
    0x3c03e4d1, 0x4b04d447, 0xd20d85fd, 0xa50ab56b, 0x35b5a8fa, 0x42b2986c, 0xdbbbc9d6, 0xacbcf940, 0x32d86ce3,
    0x45df5c75, 0xdcd60dcf, 0xabd13d59, 0x26d930ac, 0x51de003a, 0xc8d75180, 0xbfd06116, 0x21b4f4b5, 0x56b3c423,
    0xcfba9599, 0xb8bda50f, 0x2802b89e, 0x5f058808, 0xc60cd9b2, 0xb10be924, 0x2f6f7c87, 0x58684c11, 0xc1611dab,
    0xb6662d3d, 0x76dc4190, 0x01db7106, 0x98d220bc, 0xefd5102a, 0x71b18589, 0x06b6b51f, 0x9fbfe4a5, 0xe8b8d433,
    0x7807c9a2, 0x0f00f934, 0x9609a88e, 0xe10e9818, 0x7f6a0dbb, 0x086d3d2d, 0x91646c97, 0xe6635c01, 0x6b6b51f4,
    0x1c6c6162, 0x856530d8, 0xf262004e, 0x6c0695ed, 0x1b01a57b, 0x8208f4c1, 0xf50fc457, 0x65b0d9c6, 0x12b7e950,
    0x8bbeb8ea, 0xfcb9887c, 0x62dd1ddf, 0x15da2d49, 0x8cd37cf3, 0xfbd44c65, 0x4db26158, 0x3ab551ce, 0xa3bc0074,
    0xd4bb30e2, 0x4adfa541, 0x3dd895d7, 0xa4d1c46d, 0xd3d6f4fb, 0x4369e96a, 0x346ed9fc, 0xad678846, 0xda60b8d0,
    0x44042d73, 0x33031de5, 0xaa0a4c5f, 0xdd0d7cc9, 0x5005713c, 0x270241aa, 0xbe0b1010, 0xc90c2086, 0x5768b525,
    0x206f85b3, 0xb966d409, 0xce61e49f, 0x5edef90e, 0x29d9c998, 0xb0d09822, 0xc7d7a8b4, 0x59b33d17, 0x2eb40d81,
    0xb7bd5c3b, 0xc0ba6cad, 0xedb88320, 0x9abfb3b6, 0x03b6e20c, 0x74b1d29a, 0xead54739, 0x9dd277af, 0x04db2615,
    0x73dc1683, 0xe3630b12, 0x94643b84, 0x0d6d6a3e, 0x7a6a5aa8, 0xe40ecf0b, 0x9309ff9d, 0x0a00ae27, 0x7d079eb1,
    0xf00f9344, 0x8708a3d2, 0x1e01f268, 0x6906c2fe, 0xf762575d, 0x806567cb, 0x196c3671, 0x6e6b06e7, 0xfed41b76,
    0x89d32be0, 0x10da7a5a, 0x67dd4acc, 0xf9b9df6f, 0x8ebeeff9, 0x17b7be43, 0x60b08ed5, 0xd6d6a3e8, 0xa1d1937e,
    0x38d8c2c4, 0x4fdff252, 0xd1bb67f1, 0xa6bc5767, 0x3fb506dd, 0x48b2364b, 0xd80d2bda, 0xaf0a1b4c, 0x36034af6,
    0x41047a60, 0xdf60efc3, 0xa867df55, 0x316e8eef, 0x4669be79, 0xcb61b38c, 0xbc66831a, 0x256fd2a0, 0x5268e236,
    0xcc0c7795, 0xbb0b4703, 0x220216b9, 0x5505262f, 0xc5ba3bbe, 0xb2bd0b28, 0x2bb45a92, 0x5cb36a04, 0xc2d7ffa7,
    0xb5d0cf31, 0x2cd99e8b, 0x5bdeae1d, 0x9b64c2b0, 0xec63f226, 0x756aa39c, 0x026d930a, 0x9c0906a9, 0xeb0e363f,
    0x72076785, 0x05005713, 0x95bf4a82, 0xe2b87a14, 0x7bb12bae, 0x0cb61b38, 0x92d28e9b, 0xe5d5be0d, 0x7cdcefb7,
    0x0bdbdf21, 0x86d3d2d4, 0xf1d4e242, 0x68ddb3f8, 0x1fda836e, 0x81be16cd, 0xf6b9265b, 0x6fb077e1, 0x18b74777,
    0x88085ae6, 0xff0f6a70, 0x66063bca, 0x11010b5c, 0x8f659eff, 0xf862ae69, 0x616bffd3, 0x166ccf45, 0xa00ae278,
    0xd70dd2ee, 0x4e048354, 0x3903b3c2, 0xa7672661, 0xd06016f7, 0x4969474d, 0x3e6e77db, 0xaed16a4a, 0xd9d65adc,
    0x40df0b66, 0x37d83bf0, 0xa9bcae53, 0xdebb9ec5, 0x47b2cf7f, 0x30b5ffe9, 0xbdbdf21c, 0xcabac28a, 0x53b39330,
    0x24b4a3a6, 0xbad03605, 0xcdd70693, 0x54de5729, 0x23d967bf, 0xb3667a2e, 0xc4614ab8, 0x5d681b02, 0x2a6f2b94,
    0xb40bbe37, 0xc30c8ea1, 0x5a05df1b, 0x2d02ef8d,
];
