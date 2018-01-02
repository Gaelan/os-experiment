//! The paging module manages the page table
pub use self::entry::*;
use memory::{Frame, FrameAllocator, PAGE_SIZE};
use self::table::{Level4, Table};
use core::ptr::Unique;

mod entry;
mod table;

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
#[cfg_attr(feature = "cargo-clippy", allow(use_debug))]
/// Test paging features
// TODO remove this test function
pub fn test_paging<A>(allocator: &mut A)
where
    A: FrameAllocator,
{
    let mut page_table = unsafe { ActivePageTable::new() };

    let addr = 42 * 512 * 512 * 4096; // 42th P3 entry
    let page = Page::containing_address(addr);
    let frame = allocator.allocate_frame().expect("no more frames");
    println!(
        "None = {:?}, map to {:?}",
        page_table.translate(addr),
        frame
    );
    page_table.map_to(page, &frame, EntryFlags::empty(), allocator);
    println!("Some = {:?}", page_table.translate(addr));
    println!("next free frame: {:?}", allocator.allocate_frame());

    println!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });

    page_table.unmap(Page::containing_address(addr), allocator);
    println!("None = {:?}", page_table.translate(addr));

    println!("{:#x}", unsafe {
        *(Page::containing_address(addr).start_address() as *const u64)
    });
}

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

/// Current active page table
pub struct ActivePageTable {
    /// Top level (P4) page table
    p4: Unique<Table<Level4>>,
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

impl ActivePageTable {
    /// ActivePageTable constructor
    pub unsafe fn new() -> Self {
        Self {
            p4: Unique::new_unchecked(table::P4),
        }
    }

    /// Get a reference to the P4 table
    fn p4(&self) -> &Table<Level4> {
        unsafe { self.p4.as_ref() }
    }

    /// Get a mutable reference to the P4 table
    fn p4_mut(&mut self) -> &mut Table<Level4> {
        unsafe { self.p4.as_mut() }
    }

    /// Translate virtual address to physical address
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let offset = virtual_address % PAGE_SIZE;
        self.translate_page(Page::containing_address(virtual_address))
            .map(|frame| frame.number * PAGE_SIZE + offset)
    }

    /// Translate a page to a Frame, if it exists
    fn translate_page(&self, page: Page) -> Option<Frame> {
        use self::entry::HUGE_PAGE;
        let p3 = self.p4().next_table(page.p4_index());
        let huge_page = || {
            p3.and_then(|p3| {
                let p3_entry = &p3[page.p3_index()];
                // 1GiB page?
                if let Some(start_frame) = p3_entry.pointed_frame() {
                    if p3_entry.flags().contains(HUGE_PAGE) {
                        // address must be 1GiB aligned
                        assert!(start_frame.number % (ENTRY_COUNT * ENTRY_COUNT) == 0);
                        return Some(Frame {
                            number: start_frame.number + page.p2_index() * ENTRY_COUNT
                                + page.p1_index(),
                        });
                    }
                }
                if let Some(p2) = p3.next_table(page.p3_index()) {
                    let p2_entry = &p2[page.p2_index()];
                    // 2MiB page?
                    if let Some(start_frame) = p2_entry.pointed_frame() {
                        if p2_entry.flags().contains(HUGE_PAGE) {
                            // address must be 2MiB aligned
                            assert!(start_frame.number % ENTRY_COUNT == 0);
                            return Some(Frame {
                                number: start_frame.number + page.p1_index(),
                            });
                        }
                    }
                }
                None
            })
        };

        p3.and_then(|p3| p3.next_table(page.p3_index()))
            .and_then(|p2| p2.next_table(page.p2_index()))
            .and_then(|p1| p1[page.p1_index()].pointed_frame())
            .or_else(huge_page)
    }

    /// Map a Page to a Frame
    pub fn map_to<A>(&mut self, page: Page, frame: &Frame, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let p4 = self.p4_mut();
        let mut p3 = p4.next_table_create(page.p4_index(), allocator);
        let mut p2 = p3.next_table_create(page.p3_index(), allocator);
        let mut p1 = p2.next_table_create(page.p2_index(), allocator);

        //        assert!(p1[page.p1_index()].is_unused());
        p1[page.p1_index()].set(frame, flags | PRESENT);
    }

    /// Map a page to a new Frame
    pub fn map<A>(&mut self, page: Page, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let frame = allocator.allocate_frame().expect("out of memory");
        self.map_to(page, &frame, flags, allocator)
    }

    /// Identity map a frame
    pub fn identity_map<A>(&mut self, frame: &Frame, flags: EntryFlags, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        let page = Page::containing_address(frame.start_address());
        self.map_to(page, frame, flags, allocator)
    }

    /// Unmap a page
    fn unmap<A>(&mut self, page: Page, allocator: &mut A)
    where
        A: FrameAllocator,
    {
        use x86_64::instructions::tlb;
        use x86_64::VirtualAddress;
        assert!(self.translate(page.start_address()).is_some());

        let p1 = self.p4_mut()
            .next_table_mut(page.p4_index())
            .and_then(|p3| p3.next_table_mut(page.p3_index()))
            .and_then(|p2| p2.next_table_mut(page.p2_index()))
            .expect("mapping code does not support huge pages");
        //TODO check if the following expect message is correct
        let frame = p1[page.p1_index()]
            .pointed_frame()
            .expect("could not access page frame");
        p1[page.p1_index()].set_unused();

        tlb::flush(VirtualAddress(page.start_address()));

        // TODO free p(1,2,3) table if empty
        //        allocator.deallocate_frame(frame);
    }
}
