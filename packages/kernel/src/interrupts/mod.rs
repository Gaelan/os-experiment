//! Interrupt Descriptor Table and corresponding interrupt handlers
//use x86_64::structures::idt::{ExceptionStackFrame, Idt, IdtEntry};
use memory::MemoryController;
use x86_64::structures::idt::{ExceptionStackFrame, Idt};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtualAddress;
use spin::Once;

mod gdt;

/// Double fault stack index in Interrupt Stack Table
const DOUBLE_FAULT_IST_INDEX: usize = 0;

/// Task State Segment
static TSS: Once<TaskStateSegment> = Once::new();
/// Global Descriptor Table
static GDT: Once<gdt::Gdt> = Once::new();

lazy_static! {
    static ref IDT: Idt = {
        let mut idt = Idt::new();
        idt.breakpoint.set_handler_fn(handle_breakpoint);
        unsafe {
            #[cfg_attr(feature = "cargo-clippy", allow(cast_possible_truncation))]
            idt.double_fault.set_handler_fn(handle_double_fault)
            .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
        }
        idt
    };
}

/// Set up Interrupt Descriptor Table and point to interrupt handlers
pub fn init(memory_controller: &mut MemoryController) {
    use x86_64::structures::gdt::SegmentSelector;
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    let double_fault_stack = memory_controller
        .alloc_stack(1)
        .expect("could not allocate double fault stack");

    let tss = TSS.call_once(|| {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] =
            VirtualAddress(double_fault_stack.top());
        tss
    });

    let mut code_selector = SegmentSelector(0);
    let mut tss_selector = SegmentSelector(0);
    let gdt = GDT.call_once(|| {
        let mut gdt = gdt::Gdt::new();
        code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(tss));
        gdt
    });

    gdt.load();

    unsafe {
        // Update code segment register
        set_cs(code_selector);
        // load TSS
        load_tss(tss_selector);
    }

    IDT.load();
}

/// Handle a breakpoint exception
#[allow(dead_code)]
#[cfg_attr(feature = "cargo-clippy", allow(use_debug))]
extern "x86-interrupt" fn handle_breakpoint(stack_frame: &mut ExceptionStackFrame) {
    println!("\nException: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handle a double fault
#[allow(dead_code)]
#[cfg_attr(feature = "cargo-clippy", allow(use_debug))]
#[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
extern "x86-interrupt" fn handle_double_fault(
    stack_frame: &mut ExceptionStackFrame,
    _error_code: u64,
) {
    println!("\nException: DOUBLE FAULT\n{:#?}", stack_frame);
    loop {}
}
