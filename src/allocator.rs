use spin::{Mutex, MutexGuard};
use x86_64::structures::paging::{FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::VirtAddr;

use crate::allocator::bump::BumpAllocator;
use crate::allocator::linked_list::LinkedListAllocator;
use crate::mem::BootInfoFrameAllocator;

pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;


#[global_allocator]
//static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::empty();
//static GLOBAL_ALLOCATOR: Locked<BumpAllocator> = Locked::new(BumpAllocator::new());
static GLOBAL_ALLOCATOR: Locked<LinkedListAllocator> = Locked::new(LinkedListAllocator::new());

unsafe fn init_global_allocator(heap_bottom: u64, heap_size: u64) -> Result<(), MapToError<Size4KiB>> {
    GLOBAL_ALLOCATOR.lock().init(heap_bottom as usize, heap_size as usize);
    Ok(())
}

pub const HEAP_BOTTOM: u64 = 0x4444_4444_0000;
pub const HEAP_SIZE: u64 = 1024 * 1024;//1MiB 0xFFFF_FFFF;//4GiB

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
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

pub struct Locked<A> {
    inner: Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Mutex::new(inner)
        }
    }

    pub fn lock(&self) -> MutexGuard<A> {
        self.inner.lock()
    }
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
