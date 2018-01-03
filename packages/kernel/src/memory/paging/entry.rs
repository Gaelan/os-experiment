//! The entry module represents entries in the page table
use memory::Frame;
use multiboot2::{ElfSection, ELF_SECTION_ALLOCATED, ELF_SECTION_EXECUTABLE, ELF_SECTION_WRITABLE};

/// Entry represents a single entry in the page table
pub struct Entry(u64);

bitflags! {
    pub struct EntryFlags: u64 {
        #[cfg_attr(feature = "cargo-clippy", allow(identity_op))]
        /// Whether or not the page is present in memory at the moment (as opposed to swapped out)
        const PRESENT           = 1 << 0;
        /// The page can be written to
        const WRITABLE          = 1 << 1;
        /// The page can be accessed in user mode
        const USER_ACCESSIBLE   = 1 << 2;
        /// Writes go directly to memory
        const WRITE_THROUGH     = 1 << 3;
        /// Cache disabled
        const NO_CACHE          = 1 << 4;
        /// Set by CPU when accessed
        const ACCESSED          = 1 << 5;
        /// Set by CPU when written to
        const DIRTY             = 1 << 6;
        /// Must be 0 in P1 and P2, indicates 1GiB page in P3 or 2MiB page in P4
        const HUGE_PAGE         = 1 << 7;
        /// Page isn't flushed from caches on address space switch (PGE bit of CR4 register must be set)
        const GLOBAL            = 1 << 8;
        // bits 9 - 11:     usable freely by OS
        // bits 12 - 51:    physical address
        // bits 52 - 62:    usable freely by OS
        /// Disallow execution of code in this page (NXE bit in EFER register must be set)
        const NO_EXECUTE        = 1 << 63;
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
impl Entry {
    /// Checks if the current entry is filled
    pub fn is_unused(&self) -> bool {
        self.0 == 0
    }

    /// Clears the current entry
    pub fn set_unused(&mut self) {
        self.0 = 0;
    }

    /// Reads Entry flags
    pub fn flags(&self) -> EntryFlags {
        EntryFlags::from_bits_truncate(self.0)
    }

    /// Return frame pointed to, if present
    pub fn pointed_frame(&self) -> Option<Frame> {
        if self.flags().contains(PRESENT) {
            Some(Frame::containing_address(
                self.0 as usize & 0x000f_ffff_ffff_f000,
            ))
        } else {
            None
        }
    }

    /// Set page frame and frame flags
    pub fn set(&mut self, frame: &Frame, flags: EntryFlags) {
        assert_eq!(frame.start_address() & !0x000f_ffff_ffff_f000, 0);
        self.0 = (frame.start_address() as u64) | flags.bits();
    }
}

impl EntryFlags {
    /// Initialize page table entry flags according to given ELF section flags
    pub fn from_elf_section_flags(section: &ElfSection) -> Self {
        let mut flags = Self::empty();

        if section.flags().contains(ELF_SECTION_ALLOCATED) {
            flags |= PRESENT;
        }
        if section.flags().contains(ELF_SECTION_WRITABLE) {
            flags |= WRITABLE;
        }
        if !section.flags().contains(ELF_SECTION_EXECUTABLE) {
            flags |= NO_EXECUTE;
        }

        flags
    }
}
