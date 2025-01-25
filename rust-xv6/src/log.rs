use crate::sized_format;
use core::fmt::Write;
use log::{LevelFilter, Metadata, Record};

pub struct XV6Logger {
    pub(crate) max_level: LevelFilter,
}

impl log::Log for XV6Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let pid = crate::proc::current_pid().map_or_else(
                || sized_format!(16, "none"),
                |pid| sized_format!(16, "{}", pid),
            );

            writeln!(
                crate::hw::console::CONSOLE.lock(),
                "{} - (HART{} at {} pid {}) {}",
                record.level(),
                crate::hw::asm::cpuid(),
                crate::hw::asm::ticks(),
                pid,
                record.args()
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}
