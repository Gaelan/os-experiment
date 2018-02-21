//! Temporarily maps virtual addresses using the page table so that page tables can be accessed
use memory::{Frame, FrameAllocator};
use super::{ActivePageTable, Page, VirtualAddress};
use super::table::{Level1, Table};

/// Temporary page for holding page tables
pub struct TemporaryPage {
    /// Temporary page
    page: Page,
    /// Allocator that only allocates 3 frames
    allocator: TinyAllocator,
}

/// Allocator that only allocates 3 frames for the P3 P2 and P1 page tables
struct TinyAllocator([Option<Frame>; 3]);

impl TemporaryPage {
    /// TemporaryPage constructor
    pub fn new<A>(page: Page, allocator: &mut A) -> Self
    where
        A: FrameAllocator,
    {
        Self {
            page: page,
            allocator: TinyAllocator::new(allocator),
        }
    }

    /// Map the temporary page to the given frame in the active page table
    pub fn map(&mut self, frame: &Frame, active_table: &mut ActivePageTable) -> VirtualAddress {
        use super::EntryFlags;

        assert!(
            active_table.translate_page(self.page).is_none(),
            "temporary page is already mapped"
        );
        active_table.map_to(self.page, frame, EntryFlags::WRITABLE, &mut self.allocator);
        self.page.start_address()
    }

    /// Unmap the temporary page
    pub fn unmap(&mut self, active_table: &mut ActivePageTable) {
        active_table.unmap(self.page, &mut self.allocator);
    }

    /// Map the temporary page to the given page table frame in the active page table
    pub fn map_table_frame(
        &mut self,
        frame: &Frame,
        active_table: &mut ActivePageTable,
    ) -> &mut Table<Level1> {
        unsafe { &mut *(self.map(frame, active_table) as *mut Table<Level1>) }
    }
}

impl TinyAllocator {
    /// TinyAllocator constructor
    fn new<A>(allocator: &mut A) -> Self
    where
        A: FrameAllocator,
    {
        // Allocate 3 new frames for the tiny allocator to use
        let mut f = || allocator.allocate_frame();
        let frames = [f(), f(), f()];
        TinyAllocator(frames)
    }
}

impl FrameAllocator for TinyAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        for frame_option in &mut self.0 {
            if frame_option.is_some() {
                return frame_option.take();
            }
        }
        None
    }

    fn deallocate_frame(&mut self, frame: Frame) {
        for frame_option in &mut self.0 {
            if frame_option.is_none() {
                *frame_option = Some(frame);
                return;
            }
        }
        panic!("tiny allocator can hold only 3 frames.");
    }
}
