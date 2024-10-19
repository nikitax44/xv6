use core::ffi::c_int;
use core::fmt;
use spin::Mutex;

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

    pub fn backspace(&mut self) {
        // SAFETY: c is in range
        unsafe {
            consputc(0x100);
        }
    }

    pub fn puts(&mut self, s: &str) {
        s.as_bytes().iter().for_each(|&c| self.putc(c));
    }
}

impl !Sync for Console {}

pub static CONSOLE: Mutex<Console> = Mutex::new(Console {});

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.puts(s);

        Ok(())
    }
}

#[doc(hidden)]
pub struct Wrap<'a>(pub &'a Mutex<Console>);

impl fmt::Write for Wrap<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.lock().write_str(s)
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        write!($crate::printf::Wrap(&$crate::printf::CONSOLE), $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        writeln!($crate::printf::Wrap(&$crate::printf::CONSOLE), $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! dbg {
    // NOTE: We cannot use `concat!` to make a static string as a format argument
    // of `eprintln!` because `file!` could contain a `{` or
    // `$val` expression could be a block (`{ .. }`), in which case the `eprintln!`
    // will be malformed.
    () => {
        $crate::println!("[{}:{}:{}]", core::file!(), core::line!(), core::column!())
    };
    ($val:expr $(,)?) => {
        // Use of `match` here is intentional because it affects the lifetimes
        // of temporaries - https://stackoverflow.com/a/48732525/1063961
        match $val {
            tmp => {
                $crate::println!("[{}:{}:{}] {} = {:#?}",
                    core::file!(), core::line!(), core::column!(), core::stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}
