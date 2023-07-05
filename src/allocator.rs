use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use crate::panic;

pub struct DummyGlobalAlloc;

unsafe impl GlobalAlloc for DummyGlobalAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        panic!("dealloc should be never called");
    }
}

#[global_allocator]
static ALLOCATOR: DummyGlobalAlloc = DummyGlobalAlloc;