//! Table stores one table level of the page table
use core::ops::{Index, IndexMut};
use core::marker::PhantomData;
use memory::FrameAllocator;
use memory::paging::entry::*;
use memory::paging::ENTRY_COUNT;

/// Pointer to P4 table, mapped in boot.asm
#[cfg_attr(feature = "cargo-clippy", allow(inconsistent_digit_grouping))]
pub const P4: *mut Table<Level4> = 0xffff_ffff_ffff_f000 as *mut _;

/// Page table containing 512 Entries
pub struct Table<L: TableLevel> {
    /// Array of 512 page table entries
    entries: [Entry; ENTRY_COUNT],
    /// Level of page table (P4, P3, P2, or P1)
    level: PhantomData<L>,
}

impl<L> Table<L>
where
    L: TableLevel,
{
    /// Clear all table entries
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }
}

// NOTE: currently unsure how to replace Table<L::NextLevel> types with Self
#[cfg_attr(feature = "cargo-clippy", allow(use_self))]
impl<L> Table<L>
where
    L: HierarchicalLevel,
{
    /// Get address for the next level of page table
    fn next_table_address(&self, index: usize) -> Option<usize> {
        let entry_flags = self[index].flags();
        if entry_flags.contains(EntryFlags::PRESENT) && !entry_flags.contains(EntryFlags::HUGE_PAGE)
        {
            let table_address = self as *const _ as usize;
            Some((table_address << 9) | (index << 12))
        } else {
            None
        }
    }

    /// Get reference to next table level
    pub fn next_table(&self, index: usize) -> Option<&Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &*(address as *const _) })
    }

    /// Get mutable reference to next table level
    pub fn next_table_mut(&mut self, index: usize) -> Option<&mut Table<L::NextLevel>> {
        self.next_table_address(index)
            .map(|address| unsafe { &mut *(address as *mut _) })
    }

    /// Return the next table, or create a new one
    pub fn next_table_create<A>(
        &mut self,
        index: usize,
        allocator: &mut A,
    ) -> &mut Table<L::NextLevel>
    where
        A: FrameAllocator,
    {
        if self.next_table(index).is_none() {
            assert!(
                !self.entries[index].flags().contains(EntryFlags::HUGE_PAGE),
                "mapping code does not support huge pages"
            );
            let frame = allocator.allocate_frame().expect("no frames available");
            self.entries[index].set(&frame, EntryFlags::PRESENT | EntryFlags::WRITABLE);
            self.next_table_mut(index)
                .expect("next table inexplicably does not exist")
                .zero();
        }
        self.next_table_mut(index)
            .expect("next table inexplicably does not exist")
    }
}

impl<L> Index<usize> for Table<L>
where
    L: TableLevel,
{
    type Output = Entry;

    /// Get page table entry for given index
    fn index(&self, index: usize) -> &Entry {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for Table<L>
where
    L: TableLevel,
{
    /// Get mutable page table entry for given index
    fn index_mut(&mut self, index: usize) -> &mut Entry {
        &mut self.entries[index]
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
/// Trait applied to all table levels
pub trait TableLevel {}

#[cfg_attr(feature = "cargo-clippy", allow(empty_enum))]
/// Page table level 4 (P4)
pub enum Level4 {}
#[cfg_attr(feature = "cargo-clippy", allow(empty_enum))]
/// Page table level 3 (P3)
pub enum Level3 {}
#[cfg_attr(feature = "cargo-clippy", allow(empty_enum))]
/// Page table level 2 (P2)
pub enum Level2 {}
#[cfg_attr(feature = "cargo-clippy", allow(empty_enum))]
/// Page table level 1 (P1)
pub enum Level1 {}

impl TableLevel for Level4 {}
impl TableLevel for Level3 {}
impl TableLevel for Level2 {}
impl TableLevel for Level1 {}

/// Trait to provide protection from incorrectly accessing the next level page table
pub trait HierarchicalLevel: TableLevel {
    /// Next page table level (None for Level1 table)
    type NextLevel: TableLevel;
}

impl HierarchicalLevel for Level4 {
    type NextLevel = Level3;
}

impl HierarchicalLevel for Level3 {
    type NextLevel = Level2;
}

impl HierarchicalLevel for Level2 {
    type NextLevel = Level1;
}
