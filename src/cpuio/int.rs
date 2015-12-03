
pub unsafe fn inb(port: u16) -> u8 {
    // The registers for the `in` and `out` instructions are always the
    // same: `a` for value, and `d` for the port address.
    let r: u8;
    asm!("in al, dx" : "={al}"(r) : "{dx}"(port) :: "intel", "volatile");
    r
}

pub unsafe fn outb(port: u16, value: u8,) {
    asm!("out dx, al" :: "{dx}"(port), "{al}"(value) :: "intel", "volatile");
}

pub unsafe fn inw(port: u16) -> u16 {
    let r: u16;
    asm!("in ax, dx" : "={ax}"(r) : "{dx}"(port) :: "intel", "volatile");
    r
}

pub unsafe fn outw(port: u16, value: u16) {
    asm!("out dx, ax" :: "{dx}"(port), "{ax}"(value) :: "intel", "volatile");
}

pub unsafe fn inl(port: u16) -> u32 {
    let r: u32;
    asm!("in eax, dx" : "={eax}"(r) : "{dx}"(port) :: "intel", "volatile");
    r
}

pub unsafe fn outl( port: u16, value: u32) {
    asm!("out dx, eax" :: "{dx}"(port), "{eax}"(value) :: "intel", "volatile");
}
