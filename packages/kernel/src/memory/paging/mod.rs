//! The paging module manages the page table as well as remapping the kernel
pub use self::entry::*;
pub use self::mapper::Mapper;
use self::temporary_page::TemporaryPage;
use memory::{EntryFlags, Frame, FrameAllocator, PAGE_SIZE};
use multiboot2::BootInformation;
use x86_64::instructions::tlb;
use x86_64::registers::control_regs;
use core::ops::{Deref, DerefMut};

mod entry;
mod table;
mod mapper;
mod temporary_page;

/// Number of page table entries
const ENTRY_COUNT: usize = 512;

/// Physical memory address
pub type PhysicalAddress = usize;
/// Virtual memory address
pub type VirtualAddress = usize;

#[derive(Debug, Clone, Copy)]
/// A memory page
pub struct Page {
    /// Page number
    number: usize,
}

/// Page Iterator
pub struct PageIter {
    /// Start Page
    start: Page,
    /// End Page
    end: Page,
}

/// Currently active page table
pub struct ActivePageTable {
    /// Page mapper
    mapper: Mapper,
}

/// Inactive page table
pub struct InactivePageTable {
    /// Page frame containing inactive P4 table
    p4_frame: Frame,
}

impl Page {
    /// Get page that contains the given virtual address
    pub fn containing_address(address: VirtualAddress) -> Self {
        assert!(
            address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000,
            "invalid address: 0x{:x}",
            address
        );
        Self {
            number: address / PAGE_SIZE,
        }
    }

    /// Get the start address of this page
    fn start_address(&self) -> usize {
        self.number * PAGE_SIZE
    }

    /// Get an iterator over a range of Pages
    pub fn range_inclusive(start: Self, end: Self) -> PageIter {
        PageIter {
            start: start,
            end: end,
        }
    }

    /// Get index of entry to P4 table
    fn p4_index(&self) -> usize {
        (self.number >> 27) & 0o777
    }
    /// Get index of entry to P3 table
    fn p3_index(&self) -> usize {
        (self.number >> 18) & 0o777
    }
    /// Get index of entry to P2 table
    fn p2_index(&self) -> usize {
        (self.number >> 9) & 0o777
    }
    /// Get index of entry to P1 table
    #[cfg_attr(feature = "cargo-clippy", allow(identity_op))]
    fn p1_index(&self) -> usize {
        (self.number >> 0) & 0o777
    }
}

impl Iterator for PageIter {
    type Item = Page;

    fn next(&mut self) -> Option<Page> {
        if self.start.number <= self.end.number {
            let page = self.start;
            self.start.number += 1;
            Some(page)
        } else {
            None
        }
    }
}

impl Deref for ActivePageTable {
    type Target = Mapper;

    fn deref(&self) -> &Mapper {
        &self.mapper
    }
}

impl DerefMut for ActivePageTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mapper
    }
}

impl ActivePageTable {
    /// ActivePageTable constructor
    unsafe fn new() -> Self {
        Self {
            mapper: Mapper::new(),
        }
    }

    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    /// Switch to an inactive page table from the current inactive page table
    pub fn switch(&mut self, new_table: &InactivePageTable) -> InactivePageTable {
        use x86_64::PhysicalAddress;
        use x86_64::registers::control_regs;

        let old_table = InactivePageTable {
            p4_frame: Frame::containing_address(control_regs::cr3().0 as usize),
        };
        unsafe {
            control_regs::cr3_write(PhysicalAddress(new_table.p4_frame.start_address() as u64));
        }
        old_table
    }

    /// Execute f in a temporarily mapped address space
    pub fn with<F>(
        &mut self,
        table: &mut InactivePageTable,
        temporary_page: &mut TemporaryPage,
        f: F,
    ) where
        F: FnOnce(&mut Mapper),
    {
        {
            // Save original active P4 table address by reading from CR3 register
            #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
            let original_p4 = Frame::containing_address(unsafe { control_regs::cr3().0 } as usize);
            // Map temporary page to current P4 table
            let p4_table = temporary_page.map_table_frame(&original_p4.clone(), self);
            // Overwrite recursive mapping
            self.p4_mut()[511].set(
                &table.p4_frame.clone(),
                EntryFlags::PRESENT | EntryFlags::WRITABLE,
            );
            tlb::flush_all();

            // Execute f in new context
            f(self);

            // Restore original active P4 table
            p4_table[511].set(&original_p4, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            tlb::flush_all();
        }
        temporary_page.unmap(self);
    }
}

impl InactivePageTable {
    /// InactivePageTable constructor
    pub fn new(
        frame: Frame,
        active_table: &mut ActivePageTable,
        temporary_page: &mut TemporaryPage,
    ) -> Self {
        {
            let table = temporary_page.map_table_frame(&frame.clone(), active_table);
            // Clear table
            table.zero();
            // Recursively map table
            table[511].set(&frame.clone(), EntryFlags::PRESENT | EntryFlags::WRITABLE);
        }
        temporary_page.unmap(active_table);

        Self { p4_frame: frame }
    }
}

/// Remap the kernel to a different address space
#[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
pub fn remap_kernel<A>(allocator: &mut A, boot_info: &BootInformation) -> ActivePageTable
where
    A: FrameAllocator,
{
    let mut temporary_page = TemporaryPage::new(
        Page {
            number: 0xffff_ffff,
        },
        allocator,
    );

    let mut active_table = unsafe { ActivePageTable::new() };
    let mut new_table = {
        let frame = allocator
            .allocate_frame()
            .expect("could not allocate frame");
        InactivePageTable::new(frame, &mut active_table, &mut temporary_page)
    };

    active_table.with(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info
            .elf_sections_tag()
            .expect("elf memory map tag required");

        for section in elf_sections_tag.sections() {
            if !section.is_allocated() || section.size == 0 {
                // Section is not loaded in memory and is skipped
                continue;
            }

            assert_eq!(
                section.addr as usize % PAGE_SIZE,
                0,
                "sections need to be page aligned"
            );
            println!(
                "mapping section at addr: {:#x}, size: {:#x}",
                section.addr, section.size
            );

            let flags = EntryFlags::from_elf_section_flags(section);

            let start_frame = Frame::containing_address(section.start_address());
            let end_frame = Frame::containing_address(section.end_address() - 1);
            for frame in Frame::range_inclusive(start_frame, end_frame) {
                mapper.identity_map(&frame, flags, allocator);
            }
        }

        // Identity map VGA text mode buffer
        let vga_buffer_frame = Frame::containing_address(0xb_8000);
        mapper.identity_map(&vga_buffer_frame, EntryFlags::WRITABLE, allocator);

        // Identity map Multiboot info structure
        let multiboot_start = Frame::containing_address(boot_info.start_address());
        let multiboot_end = Frame::containing_address(boot_info.end_address() - 1);
        for frame in Frame::range_inclusive(multiboot_start, multiboot_end) {
            mapper.identity_map(&frame, EntryFlags::PRESENT, allocator)
        }
    });

    // Unmap old original p4 page (created in boot.asm) and use as a guard page
    let old_table = active_table.switch(&new_table);
    println!("Switched to new table");

    let old_p4_page = Page::containing_address(old_table.p4_frame.start_address());
    active_table.unmap(old_p4_page, allocator);
    println!("guard page created at {:#x}", old_p4_page.start_address());

    active_table
}
