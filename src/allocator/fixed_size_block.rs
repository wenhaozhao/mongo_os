use core::alloc::{GlobalAlloc, Layout};
use core::ptr::{NonNull, null_mut};

use crate::allocator::{align_up, Locked};

struct ListNode {
    next: Option<&'static mut ListNode>,
}

const BLOCK_SIZE: &[usize] = &[
    0x01 << 3, 0x01 << 4,
    0x01 << 5, 0x01 << 6,
    0x01 << 7, 0x01 << 8,
    0x01 << 9, 0x01 << 10,
    0x01 << 11,
];

struct FixedSizeBlock {
    blocks: [Option<&'static mut ListNode>; BLOCK_SIZE.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl FixedSizeBlock {
    pub const fn new() -> Self {
        const EMPTY: Option<&'static mut ListNode> = None;
        Self {
            blocks: [EMPTY; BLOCK_SIZE.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_size, heap_size);
    }

    fn find_block_index(&self, layout: &Layout) -> Option<usize> {
        let rquired_size = layout.size().max(layout.align());
        BLOCK_SIZE.iter().position(|&size| {
            size >= rquired_size
        })
    }

    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match &self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            _ => null_mut()
        }
    }

    unsafe fn fallback_dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        self.fallback_allocator.deallocate(NonNull::new(ptr).unwrap(), layout);
    }
}

unsafe impl GlobalAlloc for Locked<FixedSizeBlock> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        if let Some(index) = allocator.find_block_index(&layout) {
            match allocator.blocks[index].take() {
                Some(block) => {
                    allocator.blocks[index] = block.next.take();
                    return block as *mut ListNode as *mut u8;
                }
                None => {
                    let size = BLOCK_SIZE[index];
                    let align = size;
                    let layout = Layout::from_size_align(size, align).unwrap();
                    return allocator.fallback_alloc(layout);
                }
            }
        }
        allocator.fallback_alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        match allocator.find_block_index(&layout) {
            Some(index) => {
                let new_node = ListNode {
                    next: allocator.blocks[index].take()
                };
                let new_node_ptr = ptr as *mut ListNode;
                new_node_ptr.write(new_node);
                allocator.blocks[index] = Some(&mut *new_node_ptr);
                return;
            }
            None => {
                allocator.fallback_dealloc(ptr, layout);
            }
        }
    }
}