use alloc::collections::LinkedList;
use core::{mem, ptr};
use core::alloc::{GlobalAlloc, Layout};

use crate::allocator::{align_up, Locked};
use crate::println;

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        Self { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}

pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());
        let mut current = &mut self.head;
        loop {
            match current.next {
                Some(ref mut region) => {
                    if region.end_addr() < addr {
                        current = current.next.as_mut().unwrap();
                    } else if region.end_addr() == addr {
                        // merge
                        region.size += size;
                    } else if region.end_addr() > addr {
                        Self::insert_node(current, addr, size);
                        return;
                    }
                }
                None => {
                    Self::insert_node(current, addr, size);
                    return;
                }
            }
        }
    }

    unsafe fn insert_node(current: &mut ListNode, addr: usize, size: usize) {
        let mut node = ListNode::new(size);
        node.next = current.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        current.next = Some(&mut *node_ptr);
    }

    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        let mut current = &mut self.head;
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                current = current.next.as_mut().unwrap()
            }
        }
        None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;
        let excess_size = alloc_end - alloc_end;
        if excess_size > 0 && excess_size < mem::size_of::<ListNode>() {
            Err(())
        } else {
            Ok(alloc_start)
        }
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();
        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            println!("alloc (0x{:x},0x{:x})  =>  ", alloc_start, size);
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (size, _) = LinkedListAllocator::size_align(layout);
        println!("dealloc (0x{:x},0x{:x})  <=  ", ptr as usize, size);
        self.lock().add_free_region(ptr as usize, size);
    }
}