#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use bootloader::{BootInfo, entry_point};
use x86_64::VirtAddr;

use mongo_os::{allocator, mem, println};
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
        let boxed1 = alloc_test();
        let boxed2 = alloc_test();
        let boxed3 = alloc_test();
        let boxed4 = alloc_test();
        let boxed5 = alloc_test();
        let boxed6 = alloc_test();
        let boxed7 = alloc_test();
        let boxed8 = alloc_test();
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

unsafe fn alloc_test() ->Box<TestStruct> {
    let boxed = Box::new(TestStruct { a: 0x000F, b: 0x00FF, c: 0x0FFF, d: 0xFFFF });
    let ptr = (*(&boxed as *const Box<TestStruct> as *const usize)) as *const u64;
    println!("{:x} => {}", (ptr.offset(0)) as u64, *ptr.offset(0));
    println!("{:x} => {}", (ptr.offset(1)) as u64, *ptr.offset(1));
    println!("{:x} => {}", (ptr.offset(2)) as u64, *ptr.offset(2));
    println!("{:x} => {}", (ptr.offset(3)) as u64, *ptr.offset(3));
    boxed
}
/*
fn example_create_page_map_to_0xb8000(mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator) {
    let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let phys = PhysFrame::containing_address(PhysAddr::new(vga_buffer::VGA_PHYS_ADDR));
    unsafe {
        mapper.map_to(page, phys, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator).expect("Create page map error").flush();
        let ptr: *mut u64 = page.start_address().as_mut_ptr();
        ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
    }
}
 */
