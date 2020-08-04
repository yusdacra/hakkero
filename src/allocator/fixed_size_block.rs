//! Simple fixed size block allocator.
//! Falls back to a linked list allocator when it can't allocate.
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr, ptr::NonNull};

/// The block sizes to use.
///
/// The sizes must each be power of 2 because they are also used as
/// the block alignment (alignments must be always powers of 2).
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048];

struct ListNode {
    next: Option<&'static mut ListNode>,
}

pub struct Allocator {
    list_heads: [Option<&'static mut ListNode>; BLOCK_SIZES.len()],
    fallback_allocator: linked_list_allocator::Heap,
}

impl Allocator {
    /// Creates an empty `FixedSizeBlockAllocator`.
    pub const fn new() -> Self {
        Allocator {
            list_heads: [None; BLOCK_SIZES.len()],
            fallback_allocator: linked_list_allocator::Heap::empty(),
        }
    }

    /// Initialize the allocator with the given heap bounds.
    ///
    /// # Safety
    /// This function is unsafe because the caller must guarantee that the given
    /// heap bounds are valid and that the heap is unused. This method must be
    /// called only once.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.fallback_allocator.init(heap_start, heap_size);
    }

    /// Allocates using the fallback allocator.
    fn fallback_alloc(&mut self, layout: Layout) -> *mut u8 {
        match self.fallback_allocator.allocate_first_fit(layout) {
            Ok(ptr) => ptr.as_ptr(),
            Err(_) => ptr::null_mut(),
        }
    }

    /// Calculates how much of the heap is used.
    pub fn used_heap(&self) -> usize {
        fn follow_list_head<'a>(
            head: &Option<&'a mut ListNode>,
            count: usize,
        ) -> (&'a Option<&'a mut ListNode>, usize) {
            if let Some(h) = head {
                follow_list_head(&h.next, count + 1)
            } else {
                (&None, count)
            }
        }

        let mut res = self.fallback_allocator.used();
        for (i, head) in self.list_heads.iter().enumerate() {
            let (_, count) = follow_list_head(head, 0);
            res += count * BLOCK_SIZES[i];
        }
        res
    }
}

/// Choose an appropriate block size for the given layout.
///
/// Returns an index into the `BLOCK_SIZES` array.
fn list_index(layout: &Layout) -> Option<usize> {
    let required_block_size = layout.size().max(layout.align());
    BLOCK_SIZES.iter().position(|&s| s >= required_block_size)
}

unsafe impl GlobalAlloc for Locked<Allocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();
        if let Some(index) = list_index(&layout) {
            if let Some(node) = allocator.list_heads[index].take() {
                allocator.list_heads[index] = node.next.take();
                node as *mut ListNode as *mut u8
            } else {
                // No block exists in list => allocate new block
                let block_size = BLOCK_SIZES[index];
                // Only works if all block sizes are a power of 2
                let block_align = block_size;
                let layout = Layout::from_size_align(block_size, block_align).unwrap();
                allocator.fallback_alloc(layout)
            }
        } else {
            allocator.fallback_alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut allocator = self.lock();
        if let Some(index) = list_index(&layout) {
            let new_node = ListNode {
                next: allocator.list_heads[index].take(),
            };
            // Verify that block has size and alignment required for storing node
            assert!(mem::size_of::<ListNode>() <= BLOCK_SIZES[index]);
            assert!(mem::align_of::<ListNode>() <= BLOCK_SIZES[index]);
            #[allow(clippy::cast_ptr_alignment)]
            let new_node_ptr = ptr as *mut ListNode;
            new_node_ptr.write(new_node);
            allocator.list_heads[index] = Some(&mut *new_node_ptr);
        } else {
            let ptr = NonNull::new(ptr).unwrap();
            allocator.fallback_allocator.deallocate(ptr, layout);
        }
    }
}
