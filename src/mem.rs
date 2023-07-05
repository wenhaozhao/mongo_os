use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use bootloader::BootInfo;
use linked_list_allocator::LockedHeap;
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;

use crate::println;

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

#[global_allocator]
static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe fn init_global_allocator(heap_bottom: u64, heap_size: u64) -> Result<(), MapToError<Size4KiB>> {
    GLOBAL_ALLOCATOR.lock().init(heap_bottom as usize, heap_size as usize);
    Ok(())
}

pub const HEAP_BOTTOM: u64 = 0x4444_4444_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024;//1MiB 0xFFFF_FFFF;//4GiB

pub fn init_heap(mapper: &mut OffsetPageTable, frame_allocator: &mut BootInfoFrameAllocator) -> Result<(), MapToError<Size4KiB>> {
    let page_range: PageRange<Size4KiB> = Page::range(
        Page::containing_address(VirtAddr::new(HEAP_BOTTOM)),
        Page::containing_address(VirtAddr::new(HEAP_BOTTOM + HEAP_SIZE - 1u64)),
    );
    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush(); }
    }
    unsafe {
        init_global_allocator(HEAP_BOTTOM, HEAP_SIZE)
    }
}



