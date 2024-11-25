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
            writeln!(
                crate::hw::console::CONSOLE.lock(),
                "{} - {}",
                record.level(),
                record.args()
            )
            .unwrap();
        }
    }

    fn flush(&self) {}
}
