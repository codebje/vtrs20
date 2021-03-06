use zexrunner::*;

zextest::testcase! {
; <adc,sbc> hl,<bc,de,hl,sp> (38,912 cycles)
adc16:	db	0xc7		; flag mask
    tstr	0xed,0x42,0,0,0x832c,0x4f88,0xf22b,0xb339,0x7e1f,0x1563,0xd3,0x89,0x465e
    tstr	0,0x38,0,0,0,0,0,0xf821,0,0,0,0,0	; (1024 cycles)
    tstr	0,0,0,0,0,0,0,-1,-1,-1,0xd7,0,-1	; (38 cycles)
    db	    0xf8,0xb4,0xea,0xa9			        ; expected crc
    tmsg	"<adc,sbc> hl,<bc,de,hl,sp>...."
}

zextest::testcase! {
; add hl,<bc,de,hl,sp> (19,456 cycles)
add16:	db	0xc7		; flag mask
    tstr	9,0,0,0,0xc4a5,0xc4c7,0xd226,0xa050,0x58ea,0x8566,0xc6,0xde,0x9bc9
    tstr	0x30,0,0,0,0,0,0,0xf821,0,0,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,0,0,-1,-1,-1,0xd7,0,-1	; (38 cycles)
    db	    0x89,0xfd,0xb6,0x35			        ; expected crc
    tmsg	"add hl,<bc,de,hl,sp>.........."
}

zextest::testcase! {
; add ix,<bc,de,ix,sp> (19,456 cycles)
add16x:	db	0xc7		; flag mask
    tstr	0xdd,9,0,0,0xddac,0xc294,0x635b,0x33d3,0x6a76,0xfa20,0x94,0x68,0x36f5
    tstr	0,0x30,0,0,0,0,0xf821,0,0,0,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,0,-1,0,-1,-1,0xd7,0,-1	; (38 cycles)
    db	    0xc1,0x33,0x79,0x0b			        ; expected crc
    tmsg	"add ix,<bc,de,ix,sp>.........."
}

zextest::testcase! {
; add iy,<bc,de,iy,sp> (19,456 cycles)
add16y:	db	0xc7		; flag mask
    tstr	0xfd,9,0,0,0xc7c2,0xf407,0x51c1,0x3e96,0x0bf4,0x510f,0x92,0x1e,0x71ea
    tstr	0,0x30,0,0,0,0xf821,0,0,0,0,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,-1,0,0,-1,-1,0xd7,0,-1	; (38 cycles)
    db	    0xe8,0x81,0x7b,0x9e			        ; expected crc
    tmsg	"add iy,<bc,de,iy,sp>.........."
}

zextest::testcase! {
; aluop a,nn (28,672 cycles)
alu8i:	db	0xd7		; flag mask
    tstr	0xc6,0,0,0,0x09140,0x7e3c,0x7a67,0xdf6d,0x5b61,0x0b29,0x10,0x66,0x85b2
    tstr	0x38,0,0,0,0,0,0,0,0,0,0,-1,0		; (2048 cycles)
    tstr	0,-1,0,0,0,0,0,0,0,0,0xd7,0,0		; (14 cycles)
    db	    0x48,0x79,0x93,0x60			        ; expected crc
    tmsg	"aluop a,nn...................."
}

zextest::testcase! {
; aluop a,<b,c,d,e,h,l,(hl),a> (753,664 cycles)
alu8r:	db	0xd7		; flag mask
    tstr	0x80,0,0,0,0xc53e,0x573a,0x4c4d,msbt,0xe309,0xa666,0xd0,0x3b,0xadbb
    tstr	0x3f,0,0,0,0,0,0,0,0,0,0,-1,0		; (16,384 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0xd7,0,0	; (46 cycles)
    db	    0xfe,0x43,0xb0,0x16			        ; expected crc
    tmsg	"aluop a,<b,c,d,e,h,l,(hl),a>.."
}

// Illegal opcodes: alu8 on ixh/ixl/iyh/iyl
/*
zextest::testcase! {
; aluop a,<ixh,ixl,iyh,iyl> (376,832 cycles)
alu8rx:	db	0xd7		; flag mask
    tstr	0xdd,0x84,0,0,0xd6f7,0xc76e,0xaccf,0x2847,0x22dd,0xc035,0xc5,0x38,0x234b
    tstr	0x20,0x39,0,0,0,0,0,0,0,0,0,-1,0	; (8,192 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0xd7,0,0	; (46 cycles)
    db	    0xa4,0x02,0x6d,0x5a			        ; expected crc
    tmsg	"aluop a,<ixh,ixl,iyh,iyl>....."
}
*/

zextest::testcase! {
; aluop a,(<ix,iy>+1) (229,376 cycles)
alu8x:	db	0xd7		; flag mask
    tstr	0xdd,0x86,1,0,0x90b7,msbt-1,msbt-1,0x32fd,0x406e,0xc1dc,0x45,0x6e,0xe5fa
    tstr	0x20,0x38,0,0,0,1,1,0,0,0,0,-1,0	; (16,384 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0xd7,0,0		; (14 cycles)
    db		0xe8,0x49,0x67,0x6e			        ; expected crc
    tmsg	"aluop a,(<ix,iy>+1)..........."
}

zextest::testcase! {
; bit n,(<ix,iy>+1) (2048 cycles)
bitx:	db	0x53		; flag mask
    tstr	0xdd,0xcb,1,0x46,0x2075,msbt-1,msbt-1,0x3cfc,0xa79a,0x3d74,0x51,0x27,0xca14
    tstr	0x20,0,0,0x38,0,0,0,0,0,0,0x53,0,0	; (256 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0,0,0		; (8 cycles)
    db	    0xa8,0xee,0x08,0x67			        ; expected crc
    tmsg	"bit n,(<ix,iy>+1)............."
}

zextest::testcase! {
; bit n,<b,c,d,e,h,l,(hl),a> (49,152 cycles)
bitz80:	db	0x53		; flag mask
    tstr	0xcb,0x40,0,0,0x3ef1,0x9dfc,0x7acc,msbt,0xbe61,0x7a86,0x50,0x24,0x1998
    tstr	0,0x3f,0,0,0,0,0,0,0,0,0x53,0,0		; (1024 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0,-1,0		; (48 cycles)
    db	    0x7b,0x55,0xe6,0xc8			        ; expected crc
    tmsg	"bit n,<b,c,d,e,h,l,(hl),a>...."
}

// failing
zextest::testcase! {
; cpd<r> (1) (6144 cycles)
cpd1:	db	0xd7		; flag mask
    tstr	0xed,0xa9,0,0,0xc7b6,0x72b4,0x18f6,msbt+17,0x8dbd,1,0xc0,0x30,0x94a3
    tstr	0,0x10,0,0,0,0,0,0,0,10,0,-1,0		; (1024 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xa8,0x7e,0x6c,0xfa			        ; expected crc
    tmsg	"cpd<r>........................"
}

zextest::testcase! {
; <inc,dec> a (3072 cycles)
inca:	db	0xd7		; flag mask
    tstr	0x3c,0,0,0,0x4adf,0xd5d8,0xe598,0x8a2b,0xa7b0,0x431b,0x44,0x5a,0xd030
    tstr	0x01,0,0,0,0,0,0,0,0,0,0,-1,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xd1,0x88,0x15,0xa4			        ; expected crc
    tmsg	"<inc,dec> a..................."
}

zextest::testcase! {
; <inc,dec> b (3072 cycles)
incb:	db	0xd7		; flag mask
    tstr	0x04,0,0,0,0xd623,0x432d,0x7a61,0x8180,0x5a86,0x1e85,0x86,0x58,0x9bbb
    tstr	0x01,0,0,0,0,0,0,0,0,0xff00,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	0x5f,0x68,0x22,0x64			            ; expected crc
    tmsg	"<inc,dec> b..................."
}

zextest::testcase! {
; <inc,dec> bc (1536 cycles)
incbc:	db	0xd7		; flag mask
    tstr	0x03,0,0,0,0xcd97,0x44ab,0x8dc9,0xe3e3,0x11cc,0xe8a4,0x02,0x49,0x2a4d
    tstr	0x08,0,0,0,0,0,0,0,0,0xf821,0,0,0	; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xd2,0xae,0x3b,0xec			        ; expected crc
    tmsg	"<inc,dec> bc.................."
}

zextest::testcase! {
; <inc,dec> c (3072 cycles)
incc:	db	0xd7		; flag mask
    tstr	0x0c,0,0,0,0xd789,0x0935,0x055b,0x9f85,0x8b27,0xd208,0x95,0x05,0x0660
    tstr	0x01,0,0,0,0,0,0,0,0,0xff,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xc2,0x84,0x55,0x4c			        ; expected crc
    tmsg	"<inc,dec> c..................."
}

zextest::testcase! {
; <inc,dec> d (3072 cycles)
incd:	db	0xd7		; flag mask
    tstr	0x14,0,0,0,0xa0ea,0x5fba,0x65fb,0x981c,0x38cc,0xdebc,0x43,0x5c,0x03bd
    tstr	0x01,0,0,0,0,0,0,0,0xff00,0,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x45,0x23,0xde,0x10			        ; expected crc
    tmsg	"<inc,dec> d..................."
}

zextest::testcase! {
; <inc,dec> de (1536 cycles)
incde:	db	0xd7		; flag mask
    tstr	0x13,0,0,0,0x342e,0x131d,0x28c9,0x0aca,0x9967,0x3a2e,0x92,0xf6,0x9d54
    tstr	0x08,0,0,0,0,0,0,0,0xf821,0,0,0,0	; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xae,0xc6,0xd4,0x2c			        ; expected crc
    tmsg	"<inc,dec> de.................."
}

zextest::testcase! {
; <inc,dec> e (3072 cycles)
ince:	db	0xd7		; flag mask
    tstr	0x1c,0,0,0,0x602f,0x4c0d,0x2402,0xe2f5,0xa0f4,0xa10a,0x13,0x32,0x5925
    tstr	0x01,0,0,0,0,0,0,0,0xff,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xe1,0x75,0xaf,0xcc			        ; expected crc
    tmsg	"<inc,dec> e..................."
}

zextest::testcase! {
; <inc,dec> h (3072 cycles)
inch:	db	0xd7		; flag mask
    tstr	0x24,0,0,0,0x1506,0xf2eb,0xe8dd,0x262b,0x11a6,0xbc1a,0x17,0x06,0x2818
    tstr	0x01,0,0,0,0,0,0,0xff00,0,0,0,0,0	; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x1c,0xed,0x84,0x7d			        ; expected crc
    tmsg	"<inc,dec> h..................."
}

zextest::testcase! {
; <inc,dec> hl (1536 cycles)
inchl:	db	0xd7		; flag mask
    tstr	0x23,0,0,0,0xc3f4,0x07a5,0x1b6d,0x4f04,0xe2c2,0x822a,0x57,0xe0,0xc3e1
    tstr	0x08,0,0,0,0,0,0,0xf821,0,0,0,0,0	; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xfc,0x0d,0x6d,0x4a			        ; expected crc
    tmsg	"<inc,dec> hl.................."
}

zextest::testcase! {
; <inc,dec> ix (1536 cycles)
incix:	db	0xd7		; flag mask
    tstr	0xdd,0x23,0,0,0xbc3c,0x0d9b,0xe081,0xadfd,0x9a7f,0x96e5,0x13,0x85,0x0be2
    tstr	0,8,0,0,0,0,0xf821,0,0,0,0,0,0		; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xa5,0x4d,0xbe,0x31			        ; expected crc
    tmsg	"<inc,dec> ix.................."
}

zextest::testcase! {
; <inc,dec> iy (1536 cycles)
inciy:	db	0xd7		; flag mask
    tstr	0xfd,0x23,0,0,0x9402,0x637a,0x3182,0xc65a,0xb2e9,0xabb4,0x16,0xf2,0x6d05
    tstr	0,8,0,0,0,0xf821,0,0,0,0,0,0,0		; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x50,0x5d,0x51,0xa3			        ; expected crc
    tmsg	"<inc,dec> iy.................."
}

zextest::testcase! {
; <inc,dec> l (3072 cycles)
incl:	db	0xd7		; flag mask
    tstr	0x2c,0,0,0,0x8031,0xa520,0x4356,0xb409,0xf4c1,0xdfa2,0xd1,0x3c,0x3ea2
    tstr	0x01,0,0,0,0,0,0,0xff,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x56,0xcd,0x06,0xf3			        ; expected crc
    tmsg	"<inc,dec> l..................."
}

zextest::testcase! {
; <inc,dec> (hl) (3072 cycles)
incm:	db	0xd7		; flag mask
    tstr	0x34,0,0,0,0xb856,0x0c7c,0xe53e,msbt,0x877e,0xda58,0x15,0x5c,0x1f37
    tstr	0x01,0,0,0,0xff,0,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xb8,0x3a,0xdc,0xef			        ; expected crc
    tmsg	"<inc,dec> (hl)................"
}

zextest::testcase! {
; <inc,dec> sp (1536 cycles)
incsp:	db	0xd7		; flag mask
    tstr	0x33,0,0,0,0x346f,0xd482,0xd169,0xdeb6,0xa494,0xf476,0x53,0x02,0x855b
    tstr	0x08,0,0,0,0,0,0,0,0,0,0,0,0xf821	; (256 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x5d,0xac,0xd5,0x27			        ; expected crc
    tmsg	"<inc,dec> sp.................."
}

zextest::testcase! {
; <inc,dec> (<ix,iy>+1) (6144 cycles)
incx:	db	0xd7		; flag mask
    tstr	0xdd,0x34,1,0,0xfa6e,msbt-1,msbt-1,0x2c28,0x8894,0x5057,0x16,0x33,0x286f
    tstr	0x20,1,0,0,0xff,0,0,0,0,0,0,0,0		; (1024 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x20,0x58,0x14,0x70			        ; expected crc
    tmsg	"<inc,dec> (<ix,iy>+1)........."
}

/* Illegal instructions
zextest::testcase! {
; <inc,dec> ixh (3072 cycles)
incxh:	db	0xd7		; flag mask
    tstr	0xdd,0x24,0,0,0xb838,0x316c,0xc6d4,0x3e01,0x8358,0x15b4,0x81,0xde,0x4259
    tstr	0,1,0,0,0,0xff00,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x6f,0x46,0x36,0x62			        ; expected crc
    tmsg	"<inc,dec> ixh................."
}
*/

/* Illegal instructions
zextest::testcase! {
; <inc,dec> ixl (3072 cycles)
incxl:	db	0xd7		; flag mask
    tstr	0xdd,0x2c,0,0,0x4d14,0x7460,0x76d4,0x06e7,0x32a2,0x213c,0xd6,0xd7,0x99a5
    tstr	0,1,0,0,0,0xff,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x02,0x7b,0xef,0x2c			        ; expected crc
    tmsg	"<inc,dec> ixl................."
}
*/

/* Illegal instructions
zextest::testcase! {
; <inc,dec> iyh (3072 cycles)
incyh:	db	0xd7		; flag mask
    tstr	0xdd,0x24,0,0,0x2836,0x9f6f,0x9116,0x61b9,0x82cb,0xe219,0x92,0x73,0xa98c
    tstr	0,1,0,0,0xff00,0,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x2d,0x96,0x6c,0xf3			        ; expected crc
    tmsg	"<inc,dec> iyh................."
}
*/

/* Illegal instructions
zextest::testcase! {
; <inc,dec> iyl (3072 cycles)
incyl:	db	0xd7		; flag mask
    tstr	0xdd,0x2c,0,0,0xd7c6,0x62d5,0xa09e,0x7039,0x3e7e,0x9f12,0x90,0xd9,0x220f
    tstr	0,1,0,0,0xff,0,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0xfb,0xcb,0xba,0x95			        ; expected crc
    tmsg	"<inc,dec> iyl................."
}
*/

zextest::testcase! {
; cpi<r> (1) (6144 cycles)
cpi1:	db	0xd7		; flag mask
    tstr	0xed,0xa1,0,0,0x4d48,0xaf4a,0x906b,msbt,0x4e71,1,0x93,0x6a,0x907c
    tstr	0,0x10,0,0,0,0,0,0,0,010,0,-1,0		; (1024 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x06,0xde,0xb3,0x56			        ; expected crc
    tmsg	"cpi<r>........................"
}

zextest::testcase! {
; <daa,cpl,scf,ccf>
daaop:	db	0xd7		; flag mask
    tstr	0x27,0,0,0,0x2141,0x09fa,0x1d60,0xa559,0x8d5b,0x9079,0x04,0x8e,0x299d
    tstr	0x18,0,0,0,0,0,0,0,0,0,0xd7,-1,0	; (65,536 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycle)
    db	    0x9b,0x4b,0xa6,0x75			        ; expected crc
    tmsg	"<daa,cpl,scf,ccf>............."
}

zextest::testcase! {
; ld <bc,de>,(nnnn) (32 cycles)
ld161:	db	0xd7		; flag mask
    tstr	0xed,0x4b,msbtlo,msbthi,0xf9a8,0xf559,0x93a4,0xf5ed,0x6f96,0xd968,0x86,0xe6,0x4bd8
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0x4d,0x45,0xa9,0xac			        ; expected crc
    tmsg	"ld <bc,de>,(nnnn)............."
}

zextest::testcase! {
; ld hl,(nnnn) (16 cycles)
ld162:	db	0xd7		; flag mask
    tstr	0x2a,msbtlo,msbthi,0,0x9863,0x7830,0x2077,0xb1fe,0xb9fa,0xabb8,0x04,0x06,0x6015
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycle)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0x5f,0x97,0x24,0x87			        ; expected crc
    tmsg	"ld hl,(nnnn).................."
}

zextest::testcase! {
; ld sp,(nnnn) (16 cycles)
ld163:	db	0xd7		; flag mask
    tstr	0xed,0x7b,msbtlo,msbthi,0x8dfc,0x57d7,0x2161,0xca18,0xc185,0x27da,0x83,0x1e,0xf460
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0x7a,0xce,0xa1,0x1b			        ; expected crc
    tmsg	"ld sp,(nnnn).................."
}

zextest::testcase! {
; ld <ix,iy>,(nnnn) (32 cycles)
ld164:	db	0xd7		; flag mask
    tstr	0xdd,0x2a,msbtlo,msbthi,0xded7,0xa6fa,0xf780,0x244c,0x87de,0xbcc2,0x16,0x63,0x4c96
    tstr	0x20,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0x85,0x8b,0xf1,0x6d			        ; expected crc
    tmsg	"ld <ix,iy>,(nnnn)............."
}

zextest::testcase! {
; ld (nnnn),<bc,de> (64 cycles)
ld165:	db	0xd7		; flag mask
    tstr	0xed,0x43,msbtlo,msbthi,0x1f98,0x844d,0xe8ac,0xc9ed,0xc95d,0x8f61,0x80,0x3f,0xc7bf
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,0,0,0,0,-1,-1,0,0,0		    ; (32 cycles)
    db	    0x64,0x1e,0x87,0x15			        ; expected crc
    tmsg	"ld (nnnn),<bc,de>............."
}

zextest::testcase! {
; ld (nnnn),hl (16 cycles)
ld166:	db	0xd7		; flag mask
    tstr	0x22,msbtlo,msbthi,0,0xd003,0x7772,0x7f53,0x3f72,0x64ea,0xe180,0x10,0x2d,0x35e9
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycle)
    tstr	0,0,0,0,0,0,0,-1,0,0,0,0,0		    ; (16 cycles)
    db	    0xa3,0x60,0x8b,0x47			        ; expected crc
    tmsg	"ld (nnnn),hl.................."
}

zextest::testcase! {
; ld (nnnn),sp (16 cycles)
ld167:	db	0xd7		; flag mask
    tstr	0xed,0x73,msbtlo,msbthi,0xc0dc,0xd1d6,0xed5a,0xf356,0xafda,0x6ca7,0x44,0x9f,0x3f0a
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycle)
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,-1		    ; (16 cycles)
    db	    0x16,0x58,0x5f,0xd7			        ; expected crc
    tmsg	"ld (nnnn),sp.................."
}

zextest::testcase! {
; ld (nnnn),<ix,iy> (64 cycles)
ld168:	db	0xd7		; flag mask
    tstr	0xdd,0x22,msbtlo,msbthi,0x6cc3,0x0d91,0x6900,0x8ef8,0xe3d6,0xc3f7,0xc6,0xd9,0xc2df
    tstr	0x20,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,0,-1,-1,0,0,0,0,0,0		    ; (32 cycles)
    db	    0xba,0x10,0x2a,0x6b			        ; expected crc
    tmsg	"ld (nnnn),<ix,iy>............."
}

zextest::testcase! {
; ld <bc,de,hl,sp>,nnnn (64 cycles)
ld16im:	db	0xd7		; flag mask
    tstr	1,0,0,0,0x5c1c,0x2d46,0x8eb9,0x6078,0x74b1,0xb30e,0x46,0xd1,0x30cc
    tstr	0x30,0,0,0,0,0,0,0,0,0,0,0,0		; (4 cycles)
    tstr	0,0xff,0xff,0,0,0,0,0,0,0,0,0,0		; (16 cycles)
    db	    0xde,0x39,0x19,0x69			        ; expected crc
    tmsg	"ld <bc,de,hl,sp>,nnnn........."
}

zextest::testcase! {
; ld <ix,iy>,nnnn (32 cycles)
ld16ix:	db	0xd7		; flag mask
    tstr	0xdd,0x21,0,0,0x87e8,0x2006,0xbd12,0xb69b,0x7253,0xa1e5,0x51,0x13,0xf1bd
    tstr	0x20,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0xff,0xff,0,0,0,0,0,0,0,0,0		; (16 cycles)
    db	    0x22,0x7d,0xd5,0x25			        ; expected crc
    tmsg	"ld <ix,iy>,nnnn..............."
}

zextest::testcase! {
; ld a,<(bc),(de)> (44 cycles)
ld8bd:	db	0xd7		; flag mask
    tstr	0x0a,0,0,0,0xb3a8,0x1d2a,0x7f8e,0x42ac,msbt,msbt,0xc6,0xb1,0xef8e
    tstr	0x10,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0xd7,-1,0	; (22 cycles)
    db	    0xb0,0x81,0x89,0x35			        ; expected crc
    tmsg	"ld a,<(bc),(de)>.............."
}

zextest::testcase! {
; ld <b,c,d,e,h,l,(hl),a>,nn (64 cycles)
ld8im:	db	0xd7		; flag mask
    tstr	6,0,0,0,0xc407,0xf49d,0xd13d,0x0339,0xde89,0x7455,0x53,0xc0,0x5509
    tstr	0x38,0,0,0,0,0,0,0,0,0,0,0,0		; (8 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0,-1,0		    ; (8 cycles)
    db	    0xf1,0xda,0xb5,0x56			        ; expected crc
    tmsg	"ld <b,c,d,e,h,l,(hl),a>,nn...."
}

zextest::testcase! {
; ld (<ix,iy>+1),nn (32 cycles)
ld8imx:	db	0xd7		; flag mask
    tstr	0xdd,0x36,1,0,0x1b45,msbt-1,msbt-1,0xd5c1,0x61c7,0xbdc4,0xc0,0x85,0xcd16
    tstr	0x20,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,-1,0,0,0,0,0,0,0,-1,0		    ; (16 cycles)
    db	    0x26,0xdb,0x47,0x7e			        ; expected crc
    tmsg	"ld (<ix,iy>+1),nn............."
}

zextest::testcase! {
; ld <b,c,d,e>,(<ix,iy>+1) (512 cycles)
ld8ix1:	db	0xd7		; flag mask
    tstr	0xdd,0x46,1,0,0xd016,msbt-1,msbt-1,0x4260,0x7f39,0x0404,0x97,0x4a,0xd085
    tstr	0x20,0x18,0,0,0,1,1,0,0,0,0,0,0		; (32 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0xcc,0x11,0x06,0xa8			        ; expected crc
    tmsg	"ld <b,c,d,e>,(<ix,iy>+1)......"
}

zextest::testcase! {
; ld <h,l>,(<ix,iy>+1) (256 cycles)
ld8ix2:	db	0xd7		; flag mask
    tstr	0xdd,0x66,1,0,0x84e0,msbt-1,msbt-1,0x9c52,0xa799,0x49b6,0x93,0x00,0xeead
    tstr	0x20,0x08,0,0,0,1,1,0,0,0,0,0,0		; (16 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0xfa,0x2a,0x4d,0x03			        ; expected crc
    tmsg	"ld <h,l>,(<ix,iy>+1).........."
}

zextest::testcase! {
; ld a,(<ix,iy>+1) (128 cycles)
ld8ix3:	db	0xd7		; flag mask
    tstr	0xdd,0x7e,1,0,0xd8b6,msbt-1,msbt-1,0xc612,0xdf07,0x9cd0,0x43,0xa6,0xa0e5
    tstr	0x20,0,0,0,0,1,1,0,0,0,0,0,0		; (8 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,0,0		    ; (16 cycles)
    db	    0xa5,0xe9,0xac,0x64			        ; expected crc
    tmsg	"ld a,(<ix,iy>+1).............."
}

/* Illegal instructions
zextest::testcase! {
; ld <ixh,ixl,iyh,iyl>,nn (32 cycles)
ld8ixy:	db	0xd7		; flag mask
    tstr	0xdd,0x26,0,0,0x3c53,0x4640,0xe179,0x7711,0xc107,0x1afa,0x81,0xad,0x5d9b
    tstr	0x20,8,0,0,0,0,0,0,0,0,0,0,0		; (4 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0,-1,0		    ; (8 cycles)
    db	    0x24,0xe8,0x82,0x8b			        ; expected crc
    tmsg	"ld <ixh,ixl,iyh,iyl>,nn......."
}
*/

zextest::testcase! {
; ld <b,c,d,e,h,l,a>,<b,c,d,e,h,l,a> (3456 cycles)
ld8rr:	db	0xd7		; flag mask
    tstr	0x40,0,0,0,0x72a4,0xa024,0x61ac,msbt,0x82c7,0x718f,0x97,0x8f,0xef8e
    tstr	0x3f,0,0,0,0,0,0,0,0,0,0,0,0		; (64 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0xd7,-1,0	; (54 cycles)
    db	    0x74,0x4b,0x01,0x18			        ; expected crc
    tmsg	"ld <bcdehla>,<bcdehla>........"
}

/* Illegal instructions
zextest::testcase! {
; ld <b,c,d,e,ixy,a>,<b,c,d,e,ixy,a> (6912 cycles)
ld8rrx:	db	0xd7		; flag mask
    tstr	0xdd,0x40,0,0,0xbcc5,msbt,msbt,msbt,0x2fc2,0x98c0,0x83,0x1f,0x3bcd
    tstr	0x20,0x3f,0,0,0,0,0,0,0,0,0,0,0		; (128 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0xd7,-1,0	; (54 cycles)
    db	    0x47,0x8b,0xa3,0x6b			        ; expected crc
    tmsg	"ld <bcdexya>,<bcdexya>........"
}
*/

zextest::testcase! {
; ld a,(nnnn) / ld (nnnn),a (44 cycles)
lda:	db	0xd7		; flag mask
    tstr	0x32,msbtlo,msbthi,0,0xfd68,0xf4ec,0x44a0,0xb543,0x0653,0xcdba,0xd2,0x4f,0x1fd8
    tstr	0x08,0,0,0,0,0,0,0,0,0,0,0,0		; (2 cycle)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0xd7,-1,0	; (22 cycles)
    db	    0xc9,0x26,0x2d,0xe5			        ; expected crc
    tmsg	"ld a,(nnnn) / ld (nnnn),a....."
}

zextest::testcase! {
; ldd<r> (1) (44 cycles)
ldd1:	db	0xd7		; flag mask
    tstr	0xed,0xa8,0,0,0x9852,0x68fa,0x66a1,msbt+3,msbt+1,1,0xc1,0x68,0x20b7
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0xd7,0,0		; (22 cycles)
    db	    0x94,0xf4,0x27,0x69			        ; expected crc
    tmsg	"dd<r> (1)...................."
}

zextest::testcase! {
; ldd<r> (2) (44 cycles)
ldd2:	db	0xd7		; flag mask
    tstr	0xed,0xa8,0,0,0xf12e,0xeb2a,0xd5ba,msbt+3,msbt+1,2,0x47,0xff,0xfbe4
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0xd7,0,0		; (22 cycles)
    db	    0x5a,0x90,0x7e,0xd4			        ; expected crc
    tmsg	"ldd<r> (2)...................."
}

zextest::testcase! {
; ldi<r> (1) (44 cycles)
ldi1:	db	0xd7		; flag mask
    tstr	0xed,0xa0,0,0,0xfe30,0x03cd,0x6058,msbt+2,msbt,1,0x04,0x60,0x2688
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0xd7,0,0		; (22 cycles)
    db	    0x9a,0xbd,0xf6,0xb5			        ; expected crc
    tmsg	"ldi<r> (1)...................."
}

zextest::testcase! {
; ldi<r> (2) (44 cycles)
ldi2:	db	0xd7		; flag mask
    tstr	0xed,0xa0,0,0,0x4ace,0xc26e,0xb188,msbt+2,msbt,2,0x14,0x2d,0xa39f
    tstr	0,0x10,0,0,0,0,0,0,0,0,0,0,0		; (2 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0xd7,0,0		; (22 cycles)
    db	    0xeb,0x59,0x89,0x1b			        ; expected crc
    tmsg	"ldi<r> (2)...................."
}

zextest::testcase! {
; neg (16,384 cycles)
negop:	db	0xd7		; flag mask
    tstr	0xed,0x44,0,0,0x38a2,0x5f6b,0xd934,0x57e4,0xd2d6,0x4642,0x43,0x5a,0x09cc
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,-1,0		; (16,384 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0,0,0		    ; (1 cycle)
    db	    0x6a,0x3c,0x3b,0xbd			        ; expected crc
    tmsg	"neg..........................."
}

zextest::testcase! {
; <rld,rrd> (7168 cycles)
rldop:	db	0xd7		; flag mask
    tstr	0xed,0x67,0,0,0x91cb,0xc48b,0xfa62,msbt,0xe720,0xb479,0x40,0x06,0x8ae2
    tstr	0,8,0,0,0xff,0,0,0,0,0,0,0,0		; (512 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,-1,0		; (14 cycles)
    db	    0x95,0x5b,0xa3,0x26			        ; expected crc
    tmsg	"<rrd,rld>....................."
}

zextest::testcase! {
; <rlca,rrca,rla,rra> (6144 cycles)
rot8080: db	0xd7		; flag mask
    tstr	7,0,0,0,0xcb92,0x6d43,0x0a90,0xc284,0x0c53,0xf50e,0x91,0xeb,0x40fc
    tstr	0x18,0,0,0,0,0,0,0,0,0,0,-1,0		; (1024 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0xd7,0,0		; (6 cycles)
    db	    0x25,0x13,0x30,0xae			        ; expected crc
    tmsg	"<rlca,rrca,rla,rra>..........."
}

/* Illegal instructions: 0x36 is "sll" that does not exist
 * Because this test is so foundational, "sll" is implemented with a warning.
 */
zextest::testcase! {
; shift/rotate (<ix,iy>+1) (416 cycles)
rotxy:	db	0xd7		; flag mask
    tstr	0xdd,0xcb,1,6,0xddaf,msbt-1,msbt-1,0xff3c,0xdbf6,0x94f4,0x82,0x80,0x61d9
    tstr	0x20,0,0,0x38,0,0,0,0,0,0,0x80,0,0	; (32 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0x57,0,0		; (13 cycles)
    db	    0x71,0x3a,0xcd,0x81			        ; expected crc
    tmsg	"shf/rot (<ix,iy>+1)..........."
}

/* Illegal instructions: 0x30..0x37 are "sll" that do not exist
 * Because this test is so foundational, "sll" is implemented with a warning.
 */
zextest::testcase! {
; shift/rotate <b,c,d,e,h,l,(hl),a> (6784 cycles)
rotz80:	db	0xd7		; flag mask
    tstr	0xcb,0,0,0,0xcceb,0x5d4a,0xe007,msbt,0x1395,0x30ee,0x43,0x78,0x3dad
    tstr	0,0x3f,0,0,0,0,0,0,0,0,0x80,0,0		; (128 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0x57,-1,0	; (53 cycles)
    db	    0xeb,0x60,0x4d,0x58			        ; expected crc
    tmsg	"shf/rot <b,c,d,e,h,l,(hl),a>.."
}

zextest::testcase! {
; <set,res> n,<b,c,d,e,h,l,(hl),a> (7936 cycles)
srz80:	db	0xd7		; flag mask
    tstr	0xcb,0x80,0,0,0x2cd5,0x97ab,0x39ff,msbt,0xd14b,0x6ab2,0x53,0x27,0xb538
    tstr	0,0x7f,0,0,0,0,0,0,0,0,0,0,0		; (128 cycles)
    tstr	0,0,0,0,0xff,0,0,0,-1,-1,0xd7,-1,0	; (62 cycles)
    db	    0x8b,0x57,0xf0,0x08			        ; expected crc
    tmsg	"<set,res> n,<bcdehl(hl)a>....."
}

zextest::testcase! {
; <set,res> n,(<ix,iy>+1) (1792 cycles)
srzx:	db	0xd7		; flag mask
    tstr	0xdd,0xcb,1,0x86,0xfb44,msbt-1,msbt-1,0xba09,0x68be,0x32d8,0x10,0x5e,0xa867
    tstr	0x20,0,0,0x78,0,0,0,0,0,0,0,0,0	    ; (128 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0xd7,0,0		;(14 cycles)
    db	    0xcc,0x63,0xf9,0x8a			        ; expected crc
    tmsg	"<set,res> n,(<ix,iy>+1)......."
}

zextest::testcase! {
; ld (<ix,iy>+1),<b,c,d,e> (1024 cycles)
st8ix1:	db	0xd7		; flag mask
    tstr	0xdd,0x70,1,0,0x270d,msbt-1,msbt-1,0xb73a,0x887b,0x99ee,0x86,0x70,0xca07
    tstr	0x20,0x03,0,0,0,1,1,0,0,0,0,0,0		; (32 cycles)
    tstr	0,0,0,0,0,0,0,0,-1,-1,0,0,0		    ; (32 cycles)
    db	    0x04,0x62,0x6a,0xbf			        ; expected crc
    tmsg	"ld (<ix,iy>+1),<b,c,d,e>......"
}

zextest::testcase! {
; ld (<ix,iy>+1),<h,l> (256 cycles)
st8ix2:	db	0xd7		; flag mask
    tstr	0xdd,0x74,1,0,0xb664,msbt-1,msbt-1,0xe8ac,0xb5f5,0xaafe,0x12,0x10,0x9566
    tstr	0x20,0x01,0,0,0,1,1,0,0,0,0,0,0		; (16 cycles)
    tstr	0,0,0,0,0,0,0,-1,0,0,0,0,0		    ; (32 cycles)
    db	    0x6a,0x1a,0x88,0x31			        ; expected crc
    tmsg	"ld (<ix,iy>+1),<h,l>.........."
}

zextest::testcase! {
; ld (<ix,iy>+1),a (64 cycles)
st8ix3:	db	0xd7		; flag mask
    tstr	0xdd,0x77,1,0,0x67af,msbt-1,msbt-1,0x4f13,0x0644,0xbcd7,0x50,0xac,0x5faf
    tstr	0x20,0,0,0,0,1,1,0,0,0,0,0,0		; (8 cycles)
    tstr	0,0,0,0,0,0,0,0,0,0,0,-1,0		    ; (8 cycles)
    db	    0xcc,0xbe,0x5a,0x96			        ; expected crc
    tmsg	"ld (<ix,iy>+1),a.............."
}

zextest::testcase! {
; ld (<bc,de>),a (96 cycles)
stabd:	db	0xd7		; flag mask
    tstr	2,0,0,0,0x0c3b,0xb592,0x6cff,0x959e,msbt,msbt+1,0xc1,0x21,0xbde7
    tstr	0x18,0,0,0,0,0,0,0,0,0,0,0,0		; (4 cycles)
    tstr	0,0,0,0,-1,0,0,0,0,0,0,-1,0		    ; (24 cycles)
    db	    0x7a,0x4c,0x11,0x4f			        ; expected crc
    tmsg	"ld (<bc,de>),a................"
}

#[cfg(test)]
mod cpu_test {
    use std::io::{stdout, Write};
    use std::rc::Rc;

    use emulator::bus::Bus;
    use emulator::cpu::{Register, CPU};
    use emulator::ram::RAM;
    use emulator::types::Peripheral;

    struct CIO {}

    impl CIO {
        fn new() -> CIO {
            CIO {}
        }
    }

    impl Peripheral for CIO {
        fn io_write(&self, address: u16, data: u8) {
            if address == 0xff {
                print!("{}", data as char);
                stdout().flush().unwrap();
            }
        }
    }

    fn print_cpu(cpu: &CPU, bus: &mut Bus) {
        let opcodes = [
            bus.mem_read(cpu.reg(Register::PC) as u32), // assume identity MMU
            bus.mem_read(cpu.reg(Register::PC) as u32 + 1),
            bus.mem_read(cpu.reg(Register::PC) as u32 + 2),
            bus.mem_read(cpu.reg(Register::PC) as u32 + 3),
            bus.mem_read(cpu.reg(Register::PC) as u32 + 4),
            bus.mem_read(cpu.reg(Register::PC) as u32 + 5),
        ];
        let flags = cpu.reg(Register::F);
        println!(
            "PC=${:04x}, opcode=${:02x} mem={:02x} {:02x} \
                A=${:02x} BC=${:04x} DE=${:04x} HL=${:04x} IX=${:04x} IY=${:04x} \
                {}{}-{}-{}{}{}       {}",
            cpu.reg(Register::PC),
            opcodes[0],
            bus.mem_read(0x103),
            bus.mem_read(0x104),
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

    #[test]
    fn zexdoc_list_states() {
        let mut bus = Bus::new();
        let mut cpu = CPU::new(&mut bus);
        let ram = Rc::new(RAM::new(0x0000, 0x10000));
        // CPM control
        ram.write(
            0x0,
            &[
                0xC3, 0x1E, 0x00, //            JP   boot
                0x00, //                        NOP
                0x00, //                        NOP
                0x3E, 0x02, //        CPM:      LD   a,2
                0xB9, //                        CP   c
                0xCA, 0x1A, 0x00, //            JP   z,oute
                0x62, 0x6B, //                  LD   hl,de
                0x7E, //              LOOP:     LD   a,(hl)
                0xFE, 0x24, //                  CP   '$'
                0xCA, 0x1D, 0x00, //            JP   z,done
                0xED, 0x39, 0xFF, //            OUT0 (0xff), a
                0x23, //                        INC   hl
                0xC3, 0x0D, 0x00, //            JP   loop
                0xED, 0x19, 0xFF, //  OUTE:     OUT0 (0xff), e
                0xC9, //              DONE:     RET
                0x21, 0x00, 0x00, //  BOOT:     LD   hl,0
                0x36, 0x76, //                  LD   (hl),0x76 ; HALT
                0xC3, 0x00, 0x01, //            JP   0x100
            ],
        );
        ram.load_file(0x100, "tests/zexdoc.com").expect("Loading ZEXDOC test binary");

        bus.add(ram.clone());

        let cio = CIO::new();
        bus.add(Rc::new(cio));

        cpu.reset();

        // print out a trace of the first 100 opcodes
        let mut i: u64 = 0;
        while i < 100 {
            if cpu.reg(Register::PC) >= 0x100 {
                print_cpu(&cpu, &mut bus);
                i = i + 1;
            }
            cpu.cycle(&mut bus);
        }

        cpu.reset();
        println!("trace done");

        // we only want to call one test
        // so set hl to 0x1302 and jump to 0x1ae2, execute until end of stt

        cpu.write_reg(Register::HL, 0x196);
        cpu.write_reg(Register::PC, 0x1ae2);
        while cpu.reg(Register::PC) != 0x1b79 {
            // print_cpu(&cpu, &mut bus);
            cpu.cycle(&mut bus);
            if cpu.reg(Register::PC) == 0x1d42 {
                print_cpu(&cpu, &mut bus);
            }
        }
    }
}
