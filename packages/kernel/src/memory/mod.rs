//! Memory module: handles all kernel memory operations including allocating page frames and memory
pub use self::area_frame_allocator::AreaFrameAllocator;
use self::paging::{EntryFlags, Page, PhysicalAddress};
use multiboot2::BootInformation;
use {HEAP_SIZE, HEAP_START};

mod area_frame_allocator;
pub mod heap_allocator;
mod paging;

/// FrameAllocator allocates and deallocates page frames
pub trait FrameAllocator {
    /// Allocate and return a new page frame
    fn allocate_frame(&mut self) -> Option<Frame>;
    /// Deallocate the given page frame
    fn deallocate_frame(&mut self, frame: Frame);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Page frame
pub struct Frame {
    /// Page frame size
    number: usize,
}

// NOTE: much of the page frame management code depends on page frames being identical and constant in size

/// Frame Iterator
struct FrameIter {
    /// Start frame
    start: Frame,
    /// End frame
    end: Frame,
}

/// Size of each page frame
pub const PAGE_SIZE: usize = 0x1000;

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

    /// Get all frames in range from start Frame to end Frame
    fn range_inclusive(start: Self, end: Self) -> FrameIter {
        FrameIter {
            start: start,
            end: end,
        }
    }

    /// Clone this Frame
    fn clone(&self) -> Self {
        Self {
            number: self.number,
        }
    }
}

impl Iterator for FrameIter {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start <= self.end {
            let frame = self.start.clone();
            self.start.number += 1;
            Some(frame)
        } else {
            None
        }
    }
}

/// Remap the kernel and initialize the page frame allocator from ELF memory sections
pub fn init(boot_info: &BootInformation) {
    #![cfg_attr(feature = "cargo-clippy", allow(bool_comparison))]
    #![cfg_attr(feature = "cargo-clippy", allow(filter_map))]
    #![cfg_attr(feature = "cargo-clippy", allow(replace_consts))]
    assert_has_not_been_called!("memory::init must be called only once");
    let memory_map_tag = boot_info.memory_map_tag().expect("memory map tag required");
    let elf_sections_tag = boot_info
        .elf_sections_tag()
        .expect("ELF sections tag required");

    /*
    println!("Memory Areas:");
    for area in memory_map_tag.memory_areas() {
        println!(
            "\tstart: 0x{:x},\tlength: 0x{:x}",
            area.base_addr, area.length
        );
    }

    println!("Kernel Sections:");
    for section in elf_sections_tag.sections() {
        println!(
            "\taddr: 0x{:x}, size: 0x{:x}, flags: 0x{:x}",
            section.addr, section.size, section.flags
        );
    }
    */

    let kernel_start = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr)
        .min()
        .expect("elf sections tag required");
    let kernel_end = elf_sections_tag
        .sections()
        .filter(|s| s.is_allocated())
        .map(|s| s.addr + s.size)
        .max()
        .expect("elf sections tag required");

    println!(
        "kernel_start: 0x{:x} kernel_end: 0x{:x}",
        kernel_start, kernel_end
    );

    println!(
        "multiboot_start: 0x{:x} multiboot_end: 0x{:x}",
        boot_info.start_address(),
        boot_info.end_address()
    );

    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    let mut frame_allocator = AreaFrameAllocator::new(
        kernel_start as usize,
        kernel_end as usize,
        boot_info.start_address(),
        boot_info.end_address(),
        memory_map_tag.memory_areas(),
    );

    let mut active_table = paging::remap_kernel(&mut frame_allocator, boot_info);

    let heap_start_page = Page::containing_address(HEAP_START);
    let heap_end_page = Page::containing_address(HEAP_START + HEAP_SIZE - 1);

    for page in Page::range_inclusive(heap_start_page, heap_end_page) {
        active_table.map(page, EntryFlags::WRITABLE, &mut frame_allocator);
    }
}
