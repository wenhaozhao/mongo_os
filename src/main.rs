#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use alloc::vec::Vec;

use bootloader::{BootInfo, entry_point};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, mapper, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate};

use mongo_os::{allocator, mem, println, vga_buffer};
use mongo_os::mem::BootInfoFrameAllocator;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to MongoOS");
    mongo_os::init();
    let mut offset_page_table = unsafe { mem::init_offset_page_table(VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator = BootInfoFrameAllocator::init(boot_info);
    let init_heap_result = allocator::init_heap(&mut offset_page_table, &mut frame_allocator);
    match init_heap_result {
        Ok(_) => println!("Init heap OK!"),
        Err(err) => panic!("Init heap failed, {:?}", err)
    };
    // example_create_page_map_to_0xb8000(&mut offset_page_table, &mut frame_allocator);

    unsafe {
        alloc_test();
        // alloc_test();
        // alloc_test();
        // alloc_test();
        // alloc_test();
        // alloc_test();
        // alloc_test();
        // alloc_test();
    }

    mongo_os::hlt_loop()
}

/// 32 bytes
#[repr(C)]
struct TestStruct {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

unsafe fn alloc_test() {
    let boxed = Box::new(TestStruct { a: 0, b: 0, c: 0, d: 0 });
    drop(boxed);
}

fn example_create_page_map_to_0xb8000(mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator) {
    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let phys = PhysFrame::containing_address(PhysAddr::new(vga_buffer::VGA_PHYS_ADDR));
    unsafe {
        mapper.map_to(page, phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator).expect("Create page map error").flush();
        let ptr: *mut u64 = page.start_address().as_mut_ptr();
        ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }
}
