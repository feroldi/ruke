#![feature(no_std,
           lang_items,
           const_fn,
           unique,
           core_str_ext,
           iter_cmp,
           asm)]

#![no_std]

extern crate rlibc;
extern crate spin;
extern crate multiboot2;

use spin::Mutex;

use memory::{Frame, FrameAllocator, AreaFrameAllocator};
use cpuio::{Port};

#[macro_use]
mod vga_buffer;
mod memory;
mod cpuio;

#[no_mangle]
pub extern fn rust_main(multiboot_info_addr: usize) {
    vga_buffer::clear_screen();
    
    let boot_info = unsafe { multiboot2::load(multiboot_info_addr) };

    // Available memory
    let memory_map_tag = boot_info.memory_map_tag()
                                  .expect("Memory map tag required");

    // ELF sections
    let elf_sections_tag = boot_info.elf_sections_tag()
                                    .expect("Elf-sections tag required");

    // Start and ending of kernel
    let kernel_start = elf_sections_tag.sections()
                                       .map(|s| s.addr)
                                       .min()
                                       .unwrap();

    let kernel_end = elf_sections_tag.sections()
                                     .map(|s| s.addr + s.size)
                                     .max()
                                     .unwrap();

    // Start and ending of multiboot
    let multiboot_start = multiboot_info_addr;
    let multiboot_end = multiboot_start + (boot_info.total_size as usize);

    let mut frame_allocator = memory::AreaFrameAllocator::new(
        kernel_start as usize, kernel_end as usize,
        multiboot_start as usize, multiboot_end as usize,
        memory_map_tag.memory_areas());


    let frame = frame_allocator.allocate_frame();

    println!("welcome to ruke");
    println!("allocated: {:?}", frame);

    static KEYBOARD: Mutex<Port<u8>> = Mutex::new(unsafe {
        Port::new(0x60)
    });
    
    unsafe {
        // TODO: make an interface for interrupt handler
        let code = KEYBOARD.lock().read();
        println!("port 0x60: {}", code);
    }

    loop {}
}

#[lang = "eh_personality"]
extern fn eh_personality() {
}

#[lang = "panic_fmt"]
extern fn panic_fmt(fmt: core::fmt::Arguments, file: &str, line: u32) -> ! {
    println!("\n\nPANIC in {} at line {}:", file, line);
    println!("    {}", fmt);
    loop {}
}
