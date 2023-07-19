#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod vga_buffer;
pub mod gdt;
pub mod idt;
pub mod mem;
pub mod allocator;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop()
}

pub fn init() {
    gdt::init_gdt();
    idt::init_idt();
}

pub fn hlt_loop() ->!{
    loop {
        x86_64::instructions::hlt();
    }
}


