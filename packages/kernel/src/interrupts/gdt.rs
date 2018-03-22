//! Global Descriptor Table TSS setup
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::PrivilegeLevel;

/// Global Descriptor Table
pub struct Gdt {
    /// Table entries
    table: [u64; 16],
    /// Next free entry
    next_free: usize,
}

/// GDT descriptor
pub enum Descriptor {
    /// User descriptor segment
    UserSegment(u64),
    /// System descriptor segment
    SystemSegment(u64, u64),
}

bitflags! {
    ///GDT descriptor flags
    struct DescriptorFlags: u64 {
        const CONFORMING = 1 << 42;
        const EXECUTABLE = 1 << 43;
        const USER_SEGMENT = 1 << 44;
        const PRESENT = 1 << 47;
        const LONG_MODE = 1 << 53;
    }
}

impl Gdt {
    /// Gdt constructor
    pub fn new() -> Self {
        Self {
            table: [0; 16],
            next_free: 1,
        }
    }

    /// Add a new descriptor to the GDT and return a segment selector for it
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    #[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value))]
    pub fn add_entry(&mut self, entry: Descriptor) -> SegmentSelector {
        let index = match entry {
            Descriptor::UserSegment(value) => self.push(value),
            Descriptor::SystemSegment(value_low, value_high) => {
                let index = self.push(value_low);
                self.push(value_high);
                index
            }
        };
        SegmentSelector::new(index as u16, PrivilegeLevel::Ring0)
    }

    /// Push a new descriptor into the GDT descriptor table
    fn push(&mut self, value: u64) -> usize {
        if self.next_free < self.table.len() {
            let index = self.next_free;
            self.table[index] = value;
            self.next_free += 1;
            index
        } else {
            panic!("GDT full")
        }
    }

    /// Load the GDT
    #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
    pub fn load(&'static self) {
        use x86_64::instructions::tables::{lgdt, DescriptorTablePointer};
        use core::mem::size_of;

        let ptr = DescriptorTablePointer {
            base: self.table.as_ptr() as u64,
            limit: ((self.table.len() * size_of::<u64>()) - 1) as u16,
        };

        unsafe { lgdt(&ptr) };
    }
}

impl Descriptor {
    /// Create a kernel code segment descriptor
    pub fn kernel_code_segment() -> Self {
        let flags = DescriptorFlags::USER_SEGMENT | DescriptorFlags::PRESENT
            | DescriptorFlags::EXECUTABLE | DescriptorFlags::LONG_MODE;
        Descriptor::UserSegment(flags.bits())
    }

    /// Create a Task State Segment descriptor
    pub fn tss_segment(tss: &'static TaskStateSegment) -> Self {
        use core::mem::size_of;
        use bit_field::BitField;

        let ptr = tss as *const _ as u64;

        let mut low = DescriptorFlags::PRESENT.bits();

        // base address
        low.set_bits(16..40, ptr.get_bits(0..24));
        low.set_bits(56..64, ptr.get_bits(24..32));
        // limit (inclusive)
        low.set_bits(0..16, (size_of::<TaskStateSegment>() - 1) as u64);
        // Type = available 64 bit TSS
        low.set_bits(40..44, 0b1001);

        let mut high: u64 = 0;
        high.set_bits(0..32, ptr.get_bits(32..64));

        Descriptor::SystemSegment(low, high)
    }
}
