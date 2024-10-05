use crate::spinlock::Spinlock;
use core::ffi::c_int;
use core::fmt;

pub struct Console {}

extern "C" {
    fn consputc(c: c_int);
}

impl Console {
    pub fn _putc(&mut self, c: u8) {
        unsafe {
            consputc(c as c_int);
        }
    }

    pub fn puts(&mut self, s: &str) {
        s.as_bytes().iter().for_each(|&c| self._putc(c))
    }
}

pub static CONSOLE: Spinlock<Console> = Spinlock::new(Console {});

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);

        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        let _console = $crate::printf::CONSOLE.lock();
        write!(_console, $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        print!("\n");
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut _console = $crate::printf::CONSOLE.lock();
        writeln!(_console, $($arg)*).ok();
    }};
}
