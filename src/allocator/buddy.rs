use alloc::boxed::Box;
use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr;
use core::ptr::{NonNull, null_mut};

use x86_64::structures::idt::ExceptionVector::Page;
use x86_64::structures::paging::mapper::MappedFrame::Size4KiB;

use crate::{panic, print, println};
use crate::allocator::Locked;

#[derive(Debug, Copy, Clone)]
struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    const fn new() -> Self {
        LinkedList {
            head: ptr::null_mut()
        }
    }
    /// #### 入队:头插
    fn push(&mut self, node_ptr: *mut usize) {
        unsafe { *node_ptr = self.head as usize; }
        self.head = node_ptr;
        let mut x = self.head;
    }

    /// #### 出队:弹出
    fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                let ptr = self.head;
                self.head = unsafe { *ptr as *mut usize };
                Some(ptr)
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.head.is_null()
    }
}

unsafe impl Send for LinkedList {}

pub struct BuddyAllocator {
    free_lists: [LinkedList; 20],
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        BuddyAllocator {
            free_lists: [LinkedList::new(); 20] // max 256 pages (256*4kb=1mb)
        }
    }

    pub fn init(&mut self, heap_start: usize, size: usize) {
        self.add_free_region(heap_start, heap_start + size);
    }

    pub fn add_free_region(&mut self, head_start: usize, heap_end: usize) {
        let start = (head_start + size_of::<usize>() - 1) & !(size_of::<usize>() - 1);
        let end = heap_end & !(size_of::<usize>() - 1);
        assert!(start <= end);
        let mut current_start = start;
        while current_start + size_of::<usize>() <= end {
            let low_bit = current_start & !(current_start - 1);
            let size = min(low_bit, Self::prev_power_of_two(end - current_start));
            self.free_lists[size.trailing_zeros() as usize].push(current_start as *mut usize);
            current_start += size;
        }
    }

    fn prev_power_of_two(num: usize) -> usize {
        1 << (8 * size_of::<usize>() - num.leading_zeros() as usize - 1)
    }

    fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(max(layout.size().next_power_of_two(), size_of::<usize>()), layout.align());
        let target_bucket = size.trailing_zeros() as usize;
        'outer: for i in target_bucket..self.free_lists.len() {
            //println!("bucket {} is empty? {}", target_bucket, self.free_lists[i].is_empty());
            if !self.free_lists[i].is_empty() {
                for j in (target_bucket + 1..i + 1).rev() {
                    if let Some(block) = self.free_lists[j].pop() {
                        // 均分成buddy
                        self.free_lists[j - 1].push(((block as usize) + (1usize << (j - 1))) as *mut usize);
                        self.free_lists[j - 1].push(block);
                    } else {
                        return Err(());
                    }
                }
                if let Some(block) = self.free_lists[target_bucket].pop() {
                    if let Some(result) = NonNull::new(block as *mut u8) {
                        return Ok(result);
                    }
                }
                break 'outer;
            }
        }
        println!("alloc block in target bucket {} failed!", target_bucket);
        return Err(());
    }
}

unsafe impl GlobalAlloc for Locked<BuddyAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.lock().alloc(layout) {
            Ok(target) => target.as_ptr(),
            Err(()) => ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        //
    }
}