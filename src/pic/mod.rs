use cpuio::port::Port;

pub struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    pub fn handles_interrupt(&self, int_id: u8) -> bool {
        int_id >= self.offset && int_id < self.offset + 8
    }

    // end of interrupt
    pub unsafe fn eofi(&mut self) {
        self.command.write(CMD_EOFI);
    }
}

