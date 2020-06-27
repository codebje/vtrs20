use zexrunner::*;

zextest::testcase! {
; <adc,sbc> hl,<bc,de,hl,sp> (38,912 cycles)
adc16:	db	0xc7		; flag mask
    tstr	0xed,0x42,0,0,0x832c,0x4f88,0xf22b,0xb339,0x7e1f,0x1563,0xd3,0x89,0x465e
    tstr	0,0x38,0,0,0,0,0,0xf821,0,0,0,0,0	; (1024 cycles)
    tstr	0,0,0,0,0,0,0,-1,-1,-1,0xd7,0,-1	; (38 cycles)
    db		0xf8,0xb4,0xea,0xa9			; expected crc
    tmsg	"<adc,sbc> hl,<bc,de,hl,sp>...."
}

zextest::testcase! {
; aluop a,(<ix,iy>+1) (229,376 cycles)
alu8x:	db	0xd7		; flag mask
    tstr	0xdd,0x86,1,0,0x90b7,msbt-1,msbt-1,0x32fd,0x406e,0xc1dc,0x45,0x6e,0xe5fa
    tstr	0x20,0x38,0,0,0,1,1,0,0,0,0,-1,0	; (16,384 cycles)
    tstr	0,0,0,0,0xff,0,0,0,0,0,0xd7,0,0		; (14 cycles)
    db		0xe8,0x49,0x67,0x6e			; expected crc
    tmsg	"aluop a,(<ix,iy>+1)..........."
}

#[test]
fn call_that() {
    zex_adc16();
}
