//! Allocate stacks
use memory::{FrameAllocator, PAGE_SIZE};
//use memory::paging::{PageIter, ActivePageTable};
use memory::paging::{ActivePageTable, EntryFlags, Page, PageIter};

#[derive(Debug)]
/// x86_64 general stack
pub struct Stack {
    /// Stack top
    top: usize,
    /// Stack bottom
    bottom: usize,
}

/// Stack allocator
pub struct StackAllocator {
    /// Page range to allocate in
    range: PageIter,
}

impl Stack {
    /// Stack constructor
    fn new(top: usize, bottom: usize) -> Self {
        assert!(top > bottom);
        Self {
            top: top,
            bottom: bottom,
        }
    }

    /// Get top of stack
    pub fn top(&self) -> usize {
        self.top
    }

    /// Get bottom of stack
    pub fn bottom(&self) -> usize {
        self.bottom
    }
}

impl StackAllocator {
    /// StackAllocator constructor
    pub fn new(page_range: PageIter) -> Self {
        Self { range: page_range }
    }

    /// Allocate a new stack
    pub fn alloc_stack<FA: FrameAllocator>(
        &mut self,
        active_table: &mut ActivePageTable,
        frame_allocator: &mut FA,
        size_in_pages: usize,
    ) -> Option<Stack> {
        if size_in_pages == 0 {
            return None;
        }

        let mut range = self.range.clone();

        let guard_page = range.next();
        let stack_start = range.next();
        let stack_end = if size_in_pages == 1 {
            stack_start
        } else {
            range.nth(size_in_pages - 2)
        };

        match (guard_page, stack_start, stack_end) {
            (Some(_), Some(start), Some(end)) => {
                self.range = range;

                for page in Page::range_inclusive(start, end) {
                    active_table.map(page, EntryFlags::WRITABLE, frame_allocator);
                }

                let stack_top = end.start_address() + PAGE_SIZE;
                Some(Stack::new(stack_top, start.start_address()))
            }
            _ => None,
        }
    }
}
