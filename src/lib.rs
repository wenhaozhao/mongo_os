#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

pub mod vga_buffer;
pub mod gdt;
pub mod idt;
pub mod timer;
pub mod keyboard;

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt_loop()
}

pub fn init() {
    gdt::init_gdt();
    idt::init_idt();
}

pub fn halt_loop()->!{
    loop {
        x86_64::instructions::hlt();
    }
}

