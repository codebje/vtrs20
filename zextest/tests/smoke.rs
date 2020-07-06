use zexrunner::*;

zextest::testcase! {
; aluop a,nn (28,672 cycles)
alu8i:	db	0xd7		; flag mask
    tstr	0xc6,0,0,0,0x09140,0x7e3c,0x7a67,0xdf6d,0x5b61,0x0b29,0x10,0x66,0x85b2
    tstr	0x38,0,0,0,0,0,0,0,0,0,0,-1,0		; (2048 cycles)
    tstr	0,-1,0,0,0,0,0,0,0,0,0xd7,0,0		; (14 cycles)
    db	    0x48,0x79,0x93,0x60			; expected crc
    tmsg	"aluop a,nn...................."
}