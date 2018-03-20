//! The THISOSSTILLDOESNTHAVEANAMEWHATAREWEDOING kernel. As small as possible, hopefully.

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(alloc)]
#![feature(allocator_api)]
#![feature(global_allocator)]
//#![feature(const_atomic_usize_new)]
#![feature(unique)]
#![feature(ptr_internals)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(shadow_same))]
#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
#![cfg_attr(feature = "cargo-clippy", allow(unnecessary_mut_passed))]
#![cfg_attr(feature = "cargo-clippy", allow(zero_ptr))]
#![no_std]

extern crate linked_list_allocator;
extern crate multiboot2;
extern crate rlibc;
extern crate spin;
extern crate volatile;
extern crate x86_64;

#[macro_use]
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate once;

#[macro_use]
mod vga_buffer;
mod memory;

use linked_list_allocator::LockedHeap;
use alloc::boxed::Box;
use memory::heap_allocator::BumpAllocator;

/// Start of heap space
pub const HEAP_START: usize = 0o0_000_010_000_000_000;
/// Size of heap space
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
/// Global heap allocator
//static HEAP_ALLOCATOR: BumpAllocator = BumpAllocator::new(HEAP_START, HEAP_START + HEAP_SIZE);
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[no_mangle]
/// The first Rust code that runs when we boot. On x86_64, it is called from long_start.asm.
pub extern "C" fn rust_main(multiboot_information_address: usize) {
    #![cfg_attr(feature = "cargo-clippy", allow(use_debug))]
    vga_buffer::clear_screen();

    let boot_info = unsafe { multiboot2::load(multiboot_information_address) };
    enable_nxe_bit();
    enable_write_protect_bit();
    memory::init(boot_info);
    unsafe {
        HEAP_ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    let mut heap_test = Box::new(42);
    *heap_test -= 15;
    let heap_test2 = Box::new("hello");
    println!("{:?} {:?}", heap_test, heap_test2);

    let mut vec_test = vec![1, 2, 3, 4, 5, 6, 7];
    vec_test[3] = 42;
    for i in &vec_test {
        print!("{} ", i);
    }

    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}

/// Enable the NXE bit in the extended feature register (EFER) allowing the NO_EXECUTE bit to be set on pages
fn enable_nxe_bit() {
    use x86_64::registers::msr::{rdmsr, wrmsr, IA32_EFER};
    let nxe_bit = 1 << 11;
    unsafe {
        let efer = rdmsr(IA32_EFER);
        wrmsr(IA32_EFER, efer | nxe_bit);
    }
}

/// Enable the write protect bit, so that the kernel can not write to pages not flagged as WRITABLE
fn enable_write_protect_bit() {
    use x86_64::registers::control_regs::{Cr0, cr0, cr0_write};
    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}

#[lang = "eh_personality"]
/// The Rust compiler requires this for exception handling. Currently a no-op.
extern "C" fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
/// The Rust compiler requires this for panic handling. Currently just loops forever.
pub extern "C" fn panic_fmt(fmt: core::fmt::Arguments, file: &'static str, line: u32) -> ! {
    println!("PANIC in {} at line {}:", file, line);
    println!("\t{}", fmt);
    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}
