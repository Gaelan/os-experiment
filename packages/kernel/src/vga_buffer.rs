//! This module handles VGA text mode writing, used for displaying text during early boot.

extern crate spin;
extern crate volatile;

use volatile::Volatile;
use core::ptr::Unique;
use core::fmt;
use core::fmt::Write;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
#[repr(u8)]
/// A single color supported by VGA text mode.
pub enum Color {
    /// The black color.
    Black = 0,
    /// The blue color.
    Blue = 1,
    /// The green color.
    Green = 2,
    /// The cyan color.
    Cyan = 3,
    /// The red color.
    Red = 4,
    /// The magenta color.
    Magenta = 5,
    /// The brown color.
    Brown = 6,
    /// The light gray color.
    LightGray = 7,
    /// The dark gray color.
    DarkGray = 8,
    /// The light blue color.
    LightBlue = 9,
    /// The light green color.
    LightGreen = 10,
    /// The light cyan color.
    LightCyan = 11,
    /// The light red color.
    LightRed = 12,
    /// The pink color.
    Pink = 13,
    /// The yellow color.
    Yellow = 14,
    /// The white color.
    White = 15,
}

#[derive(Debug, Clone, Copy)]
/// A VGA color code, consisting of a foreground and background color.
struct ColorCode(u8);

impl ColorCode {
    /// Create a color code from a background and foreground color.
    const fn new(foreground: Color, background: Color) -> Self {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
/// Text mode VGA character
struct ScreenChar {
    /// The ASCII character to display (VGA text mode is ASCII-only).
    ascii_character: u8,
    /// The color (background and foreground) of the character.
    color_code: ColorCode,
}

/// Width of VGA text mode buffer
const BUFFER_HEIGHT: usize = 25;
/// Height of VGA text mode buffer
const BUFFER_WIDTH: usize = 80;

/// Text mode VGA screen buffer
struct Buffer {
    /// 2D Array of ScreenChars
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// VGA text mode writer. Handles wrapping and writing to the actual buffer.
pub struct Writer {
    /// The column (horizontal) position where the next character will be
    /// printed If it is ≥ `BUFFER_WIDTH`, the next character will be on a
    /// new line.
    column_position: usize,
    /// The color (background and foreground) we are currently printing in.
    color_code: ColorCode,
    /// The underlying VGA text mode buffer the writer uses to display text.
    buffer: Unique<Buffer>,
}

impl Writer {
    /// Draw a byte in the current position on the screen, moving down a line
    /// if necessary.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer().chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Gets a mutable reference to the underlying text buffer. Unsafely
    /// implemented due to the raw memory reference, but has a safe interface
    /// as long as we are constructed with the right buffer pointer (the only
    /// time this struct is constructed is in this file, so that is always
    /// true).
    fn buffer(&mut self) -> &mut Buffer {
        unsafe { self.buffer.as_mut() }
    }

    /// Draws a new line by pushing everything up a line and clearing the bottom row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_HEIGHT {
                let buffer = self.buffer();
                let character = buffer.chars[row][col].read();
                buffer.chars[row - 1][col].write(character);
            }
        }

        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Clears a row of the VGA text buffer. Used for pushing everything up a
    /// line when wrapping.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer().chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte)
        }
        Ok(())
    }
}

/// The global VGA text mode writer. Must be wrapped in a spin lock because
/// things will go badly if we try and write from more than one thread at once,
/// and we don't have a better locking mechanism yet.
pub static WRITER: spin::Mutex<Writer> = spin::Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::LightGreen, Color::Black),
    buffer: unsafe { Unique::new_unchecked(0xb_8000 as *mut _) },
});

macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::vga_buffer::_print(format_args!($($arg)*));
    });
}

/// Helper for the print macro. Don't call from outside
pub fn _print(args: fmt::Arguments) {
    #[cfg_attr(feature = "cargo-clippy", allow(result_unwrap_used))]
    WRITER.lock().write_fmt(args).unwrap();
}

macro_rules! println {
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

/// Clear the screen in VGA text mode.
pub fn clear_screen() {
    for _ in 0..BUFFER_HEIGHT {
        WRITER.lock().new_line();
    }
}
