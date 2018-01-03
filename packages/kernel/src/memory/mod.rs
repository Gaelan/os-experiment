//! Memory module: handles all kernel memory operations including allocating page frames and memory
pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::PhysicalAddress;


mod area_frame_allocator;
mod paging;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Page frame
pub struct Frame {
    /// Page frame size
    number: usize,
}

// NOTE: much of the page frame management code depends on page frames being identical and constant in size

/// Size of each page frame
pub const PAGE_SIZE: usize = 4096;

impl Frame {
    /// Set frame to correspond to physical address
    fn containing_address(address: usize) -> Self {
        Self {
            number: address / PAGE_SIZE,
        }
    }

    /// Get physical address of start of page frame
    fn start_address(&self) -> PhysicalAddress {
        self.number * PAGE_SIZE
    }

    /// Clone this Frame
    fn clone(&self) -> Self {
        Self {
            number: self.number,
        }
    }
}

/// FrameAllocator allocates and deallocates page frames
pub trait FrameAllocator {
    /// Allocate and return a new page frame
    fn allocate_frame(&mut self) -> Option<Frame>;
    /// Deallocate the given page frame
    fn deallocate_frame(&mut self, frame: Frame);
}
