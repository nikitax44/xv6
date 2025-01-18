use crate::util::Mutex;
use core::ffi::{c_int, CStr};
use core::fmt;

pub struct Console {}

extern "C" {
    /// if c is in range `0..=0xff` will put this byte to screen
    /// if c == 0x100 will put `\b \b` aka backspace to screen
    /// # Safety:
    /// `c` must be in range `0..=0x100`
    fn consputc(c: c_int);
}

impl Console {
    pub fn putc(&mut self, c: u8) {
        // SAFETY: c is in range `0..=0xff`
        unsafe {
            consputc(c.into());
        }
    }

    pub fn newline(&mut self) {
        self.putc(b'\n');
    }

    pub fn backspace(&mut self) {
        // SAFETY: c is in range
        unsafe {
            consputc(0x100);
        }
    }

    pub fn puts(&mut self, s: &str) {
        s.as_bytes().iter().for_each(|&c| self.putc(c));
    }

    pub fn putcs(&mut self, s: &CStr) {
        s.to_bytes().iter().for_each(|&c| self.putc(c));
    }

    /// # Safety
    /// actually always safe, but can lead to unreadable mess
    #[must_use]
    pub const unsafe fn get_async() -> Self {
        Self {}
    }
}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console {});

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);

        Ok(())
    }
}
