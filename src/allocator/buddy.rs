use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr;
use core::ptr::NonNull;

use crate::allocator::Locked;
use crate::println;

#[derive(Debug, Copy, Clone)]
struct LinkedList {
    head: *mut usize,
}

const LAYOUT: usize = size_of::<usize>() << 1;

impl LinkedList {
    const fn new() -> Self {
        LinkedList {
            head: ptr::null_mut() // [next,pre]
        }
    }
    /// #### 入队:头插
    unsafe fn push(&mut self, node_ptr: *mut usize) {
        if node_ptr != ptr::null_mut() {
            if self.head != ptr::null_mut() {
                self.head.offset(1).write(node_ptr as usize);
            }
            node_ptr.write(self.head as usize);
            node_ptr.offset(1).write(0usize);
            self.head = node_ptr;
        }
    }

    /// #### 出队:弹出
    unsafe fn pop(&mut self) -> Option<*mut usize> {
        match self.is_empty() {
            true => None,
            false => {
                let ptr = self.head;
                self.head = *ptr as *mut usize;
                Some(ptr)
            }
        }
    }

    unsafe fn remove(&mut self, node_ptr: *const usize) {
        if node_ptr == self.head {
            self.pop();
        } else {
            let (next, pre) = (*node_ptr, *node_ptr.offset(1) as *mut usize);
            pre.write(next);
        }
    }

    fn is_empty(&self) -> bool {
        self.head.is_null()
    }
}

unsafe impl Send for LinkedList {}

pub struct BuddyAllocator<const ORDER: usize> {
    free_lists: [LinkedList; ORDER],
}

impl<const ORDER: usize> BuddyAllocator<ORDER> {
    pub const fn new() -> Self {
        BuddyAllocator {
            free_lists: [LinkedList::new(); ORDER]
        }
    }

    pub fn init(&mut self, heap_start: usize, size: usize) {
        self.add_free_region(heap_start, heap_start + size);
    }

    pub fn add_free_region(&mut self, head_start: usize, heap_end: usize) {
        let start = (head_start + LAYOUT - 1) & !(LAYOUT - 1);
        let end = heap_end & !(LAYOUT - 1);
        assert!(start <= end);
        let mut current_start = start;
        while current_start + LAYOUT <= end {
            let size = Self::prev_power_of_two(end - current_start);
            unsafe { self.free_lists[size.trailing_zeros() as usize].push(current_start as *mut usize) };
            current_start += size;
        }
    }

    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(max(layout.size().next_power_of_two(), LAYOUT), layout.align());
        let bucket = size.trailing_zeros() as usize;

        'outer: for i in bucket..self.free_lists.len() {
            if !self.free_lists[i].is_empty() {
                for j in (bucket + 1..=i).rev() {
                    if let Some(block) = self.free_lists[j].pop() {
                        // 均分成buddy
                        self.free_lists[j - 1].push(((block as usize) + (1usize << (j - 1))) as *mut usize);
                        self.free_lists[j - 1].push(block);
                    } else {
                        return Err(());
                    }
                }
                if let Some(block) = self.free_lists[bucket].pop() {
                    if let Some(result) = NonNull::new(block as *mut u8) {
                        return Ok(result);
                    }
                }
                break 'outer;
            }
        }
        println!("alloc block in bucket {} failed!", bucket);
        return Err(());
    }

    unsafe fn dealloc(&mut self, ptr: *mut usize, layout: Layout) {
        let mut mut_ptr = ptr;
        let size = max(max(layout.size().next_power_of_two(), LAYOUT), layout.align());
        let mut bucket = size.trailing_zeros() as usize;
        for i in bucket..self.free_lists.len() {
            bucket = i;
            let mut block = self.free_lists[i].head;
            let mut continue_merge = false;
            while !block.is_null() {
                //验证 ptr/block 是否是buddy关系
                if (((mut_ptr as usize) ^ (block as usize)).trailing_zeros() as usize) == i {
                    // 从当前bucket摘掉block
                    self.free_lists[i].remove(block);
                    mut_ptr = min(mut_ptr, block);
                    continue_merge = true;
                    break;
                } else {
                    block = *block as *mut usize;
                }
            }
            if !continue_merge {
                break;
            }
        }
        self.free_lists[bucket].push(mut_ptr);
        println!("push => {} : {:x}", bucket, mut_ptr as usize);
        return;
    }

    fn prev_power_of_two(num: usize) -> usize {
        1 << (8 * size_of::<usize>() - num.leading_zeros() as usize - 1)
    }
}

unsafe impl<const ORDER: usize> GlobalAlloc for Locked<BuddyAllocator<ORDER>> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.lock().alloc(layout) {
            Ok(target) => target.as_ptr(),
            Err(()) => ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.lock().dealloc(ptr as usize as *mut usize, layout);
        println!("dealloc => {:x} ({})", ptr as usize, layout.size());
    }
}