use cpuio::int::{inb, outb, inw, outw, inl, outl};

pub struct Port<T: InOut> {
    port: u16,
    mark: ::core::marker::PhantomData<T>,
}

pub unsafe trait InOut {
    unsafe fn int_in(port: u16) -> Self;
    unsafe fn int_out(port: u16, value: Self);
}

impl<T: InOut> Port<T> {
    pub const unsafe fn new(port: u16) -> Port<T> {
        Port {
            port: port,
            mark: ::core::marker::PhantomData,
        }
    }

    pub unsafe fn read(&self) -> T {
        T::int_in(self.port)
    }

    pub unsafe fn write(&self, value: T) {
        T::int_out(self.port, value)
    }
}

unsafe impl InOut for u8 {
    unsafe fn int_in(port: u16) -> u8 { inb(port) }
    unsafe fn int_out(port: u16, value: u8) { outb(port, value) }
}

unsafe impl InOut for u16 {
    unsafe fn int_in(port: u16) -> u16 { inw(port) }
    unsafe fn int_out(port: u16, value: u16) { outw(port, value) }
}

unsafe impl InOut for u32 {
    unsafe fn int_in(port: u16) -> u32 { inl(port) }
    unsafe fn int_out(port: u16, value: u32) { outl(port, value) }
}
