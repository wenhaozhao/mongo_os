#![no_std]
#![no_main]

use bootloader::{BootInfo, entry_point};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, mapper, Mapper, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB, Translate};

use mongo_os::{mem, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to MongoOS");
    println!("Memory Length: {}", boot_info.memory_map.len());
    mongo_os::init();
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    unsafe {
        let mut mapper = mem::init_offset_page_table(VirtAddr::new(boot_info.physical_memory_offset));
        let page: Page<Size4KiB> = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
        let frame = PhysFrame::containing_address(PhysAddr::new(mongo_os::vga_buffer::VGA_PHYS_ADDR));
        let mut frame_allocator = mem::BootInfoFrameAllocator::init(boot_info);
        let r = mapper.map_to(
            page, frame,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            &mut frame_allocator,
        ).expect("map_to failed").flush();
        let ptr = page.start_address().as_mut_ptr() as *mut u64;
        ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e);
        let addrs = [0xb8000, 0x201008, 0x0100_0020_1a10, physical_memory_offset.as_u64()];
        for addr in addrs {
            let virt = VirtAddr::new(addr);
            let phys = mapper.translate_addr(virt).expect("Translate failed");
            println!("    arr_ptr: {:?} => {:?}", virt, phys);
        }
    }

    mongo_os::hlt_loop()
}


