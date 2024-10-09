use crate::util::spinlock::Spinlock;
use core::ffi::c_int;
use core::fmt;

pub struct Console {}

extern "C" {
    fn consputc(c: c_int);
}

impl Console {
    pub fn _putc(&mut self, c: u8) {
        // SAFETY:
        // c is byte.
        unsafe {
            consputc(c.into());
        }
    }

    pub fn puts(&mut self, s: &str) {
        s.as_bytes().iter().for_each(|&c| self._putc(c));
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
        use core::fmt::Write;
        let mut _console = $crate::printf::CONSOLE.lock();
        write!(_console, $($arg)*).ok();
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n");
    };
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut _console = $crate::printf::CONSOLE.lock();
        writeln!(_console, $($arg)*).ok();
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
