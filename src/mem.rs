use bootloader::BootInfo;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PhysFrame, Size4KiB};

pub unsafe fn init_offset_page_table(phys_mem_offset: VirtAddr) -> OffsetPageTable<'static> {
    use x86_64::structures::paging::PageTable;
    let (level_4_table_frame, _) = x86_64::registers::control::Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let level_4_table = &mut *(virt.as_mut_ptr() as *mut PageTable);
    OffsetPageTable::new(level_4_table, phys_mem_offset)
}

pub struct BootInfoFrameAllocator {
    boot_info: &'static BootInfo,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub fn init(boot_info: &'static BootInfo) -> BootInfoFrameAllocator {
        BootInfoFrameAllocator {
            boot_info: boot_info,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
        use bootloader::bootinfo::MemoryRegionType;
        let regions = self.boot_info.memory_map.iter();
        regions.filter(|r| r.region_type == MemoryRegionType::Usable)
            .map(|r| {
                let range = r.range;
                range.start_addr()..range.end_addr()
            })
            .flat_map(|r| r.step_by(4096))
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}



