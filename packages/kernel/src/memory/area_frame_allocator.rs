//! AreaFrameAllocator allocates page frames sequentially, without freeing them.
use memory::{Frame, FrameAllocator};
use multiboot2::{MemoryArea, MemoryAreaIter};

/// AreaFrameAllocator allocates page frames sequentially, avoiding kernel and multiboot info struct
pub struct AreaFrameAllocator {
    /// The next frame that can be allocated in the current memory area, set to none if there is no space left
    next_free_frame: Frame,
    /// The current memory area page frames are allocated in
    current_area: Option<&'static MemoryArea>,
    /// Iterator of all memory areas found by kernel using the multiboot info structure
    areas: MemoryAreaIter,
    /// Frame where the start of the kernel is loaded
    kernel_start: Frame,
    /// Frame where the end of the kernel is loaded
    kernel_end: Frame,
    /// Frame where the start of the multiboot info structure is stored
    multiboot_start: Frame,
    /// Frame where the end of the multiboot info structure is stored
    multiboot_end: Frame,
}

impl FrameAllocator for AreaFrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame> {
        if let Some(area) = self.current_area {
            // "Clone" the frame to return it if it's free. Frame doesn't
            // implement Clone, but we can construct an identical frame.
            let frame = Frame {
                number: self.next_free_frame.number,
            };

            // the last frame of the current area
            #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
            let current_area_last_frame = {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(address as usize)
            };

            if frame > current_area_last_frame {
                // all frames of current area are used, switch to next area
                self.choose_next_area();
            } else if frame >= self.kernel_start && frame <= self.kernel_end {
                // `frame` is used by the kernel
                self.next_free_frame = Frame {
                    number: self.kernel_end.number + 1,
                };
            } else if frame >= self.multiboot_start && frame <= self.multiboot_end {
                // `frame` is used by the multiboot information structure
                self.next_free_frame = Frame {
                    number: self.multiboot_end.number + 1,
                };
            } else {
                // frame is unused, increment `next_free_frame` and return it
                self.next_free_frame.number += 1;
                return Some(frame);
            }
            // `frame` was not valid, try it again with the updated `next_free_frame`
            self.allocate_frame()
        } else {
            None // no free frames left
        }
    }

    fn deallocate_frame(&mut self, _frame: Frame) {
        unimplemented!()
    }
}

impl AreaFrameAllocator {
    /// AreaFrameAllocator constructor
    pub fn new(
        kernel_start: usize,
        kernel_end: usize,
        multiboot_start: usize,
        multiboot_end: usize,
        memory_areas: MemoryAreaIter,
    ) -> Self {
        let mut allocator = Self {
            next_free_frame: Frame::containing_address(0),
            current_area: None,
            areas: memory_areas,
            kernel_start: Frame::containing_address(kernel_start),
            kernel_end: Frame::containing_address(kernel_end),
            multiboot_start: Frame::containing_address(multiboot_start),
            multiboot_end: Frame::containing_address(multiboot_end),
        };
        allocator.choose_next_area();
        allocator
    }

    /// Finds next area with free space for page frames
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    fn choose_next_area(&mut self) {
        self.current_area = self.areas
            .clone()
            .filter(|area| {
                let address = area.base_addr + area.length - 1;
                Frame::containing_address(address as usize) >= self.next_free_frame
            })
            .min_by_key(|area| area.base_addr);

        if let Some(area) = self.current_area {
            let start_frame = Frame::containing_address(area.base_addr as usize);
            if self.next_free_frame < start_frame {
                self.next_free_frame = start_frame;
            }
        }
    }
}
