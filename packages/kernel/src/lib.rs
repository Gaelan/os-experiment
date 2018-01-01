//! The THISOSSTILLDOESNTHAVEANAMEWHATAREWEDOING kernel. As small as possible, hopefully.

#![feature(lang_items)]
#![feature(const_fn)]
#![feature(unique)]
#![cfg_attr(feature = "cargo-clippy", deny(clippy))]
#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(shadow_same))]
#![cfg_attr(feature = "cargo-clippy", allow(doc_markdown))]
#![no_std]

extern crate rlibc;
extern crate spin;
extern crate volatile;

#[macro_use]
mod vga_buffer;

#[no_mangle]
/// The first Rust code that runs when we boot. On x86_64, it is called from long_start.asm.
pub extern "C" fn rust_main() {
    // ATTENTION: we have a very small stack and no guard page
    vga_buffer::clear_screen();
    println!("Foo\tBar\tBaz");
    println!("1\t2\t3");
    println!("4\t5\t6");

    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}

#[lang = "eh_personality"]
/// The Rust compiler requires this for exception handling. Currently a no-op.
extern "C" fn eh_personality() {}
#[lang = "panic_fmt"]
#[no_mangle]
/// The Rust compiler requires this for panic handling. Currently just loops forever.
pub extern "C" fn panic_fmt() -> ! {
    #[cfg_attr(feature = "cargo-clippy", allow(empty_loop))]
    loop {}
}
