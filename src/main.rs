#![no_std]
#![no_main]

use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::{mapper, PageTable, Translate};
use x86_64::VirtAddr;

use mongo_os::{mem, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Welcome to MongoOS");
    mongo_os::init();
    let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
    unsafe {
        let mapper = mem::init_offset_page_table(VirtAddr::new(boot_info.physical_memory_offset));
        let addrs = [0xb8000, 0x201008, 0x0100_0020_1a10, physical_memory_offset.as_u64()];
        for addr in addrs {
            let virt = VirtAddr::new(addr);
            let phys = mapper.translate_addr(virt).expect("Translate failed");
            println!("    arr_ptr: {:?} => {:?}", virt, phys);
        }
    }


    mongo_os::hlt_loop()
}


