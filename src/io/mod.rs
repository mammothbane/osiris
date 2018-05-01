pub use self::scan_code::ScanCode;

mod scan_code;

#[inline]
pub unsafe fn inb(port: u16) -> u8 {
    let ret: u8;

    asm!(
        "inb $1, $0"
        : "={al}"(ret)
        : "{dx}"(port)
        :
        : "volatile"
    );

    ret
}

#[inline]
pub unsafe fn outb(port: u16, val: u8) {
    asm!(
        "outb $0, $1"
        :
        : "{al}"(val), "{dx}"(port)
        :
        : "volatile"
    );
}
