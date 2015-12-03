use cpuio::int::{inb, outb, inw, outw, inl, outl};

pub struct Port<T: Int> {
    port: u16,
    mark: ::core::marker::PhantomData<T>,
}

pub struct UnsafePort<T: Int> {
    port: u16,
    mark: ::core::marker::PhantomData<T>,
}

pub trait Int {
    fn int_in(port: u16) -> Self;
    fn int_out(port: u16, value: Self);
}

impl<T: Int> Port<T> {
    pub const unsafe fn new(port: u16) -> Port<T> {
        Port {
            port: port,
            mark: ::core::marker::PhantomData,
        }
    }

    pub fn read(&self) -> T {
        T::int_in(self.port)
    }

    pub fn write(&self, value: T) {
        T::int_out(self.port, value)
    }
}

impl<T: Int> UnsafePort<T> {
    pub const unsafe fn new(port: u16) -> UnsafePort<T> {
        UnsafePort {
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

impl Int for u8 {
    fn int_in(port: u16) -> u8 { unsafe { inb(port) }}
    fn int_out(port: u16, value: u8) { unsafe { outb(port, value) }}
}

impl Int for u16 {
    fn int_in(port: u16) -> u16 { unsafe { inw(port) }}
    fn int_out(port: u16, value: u16) { unsafe { outw(port, value) }}
}

impl Int for u32 {
    fn int_in(port: u16) -> u32 { unsafe { inl(port) }}
    fn int_out(port: u16, value: u32) { unsafe { outl(port, value) }}
}
