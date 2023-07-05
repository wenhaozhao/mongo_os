#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use bootloader::{BootInfo, entry_point};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, mapper, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate};

use mongo_os::{mem, println, vga_buffer};
use mongo_os::mem::BootInfoFrameAllocator;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to MongoOS");
    mongo_os::init();
    let mut offset_page_table = unsafe { mem::init_offset_page_table(VirtAddr::new(boot_info.physical_memory_offset)) };
    let mut frame_allocator = BootInfoFrameAllocator::init(boot_info);
    let init_heap_result = mem::init_heap(&mut offset_page_table, &mut frame_allocator);
    match init_heap_result {
        Ok(_) => println!("Init heap OK!"),
        Err(err) => panic!("Init heap failed, {:?}", err)
    };
    // example_create_page_map_to_0xb8000(&mut offset_page_table, &mut frame_allocator);
    let x = Box::new("x");
    println!("{} => {:p}", &x, x);

    unsafe {
        let x_ref = x.as_ref();
        let virt = VirtAddr::from_ptr(x_ref);
        let phys = offset_page_table.translate_addr(virt);
        println!("{:o}:{:x} => {:?}",
                 virt.as_u64(),u64::from(virt.page_offset()),
                 phys
        );
    }
    let y = Box::new("y");
    println!("{} => {:p}", &y, y);
    unsafe {
        let y_ref = y.as_ref();
        let virt = VirtAddr::from_ptr(y_ref);
        let phys = offset_page_table.translate_addr(virt);
        println!("{:o}:{:x} => {:?}",
                 virt.as_u64(),u64::from(virt.page_offset()),
                 phys
        );
    }

    let y = Box::new("y");
    println!("{} => {:p}", &y, y);
    let z = Box::new("z");
    println!("{} => {:p}", &z, z);

    mongo_os::hlt_loop()
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
