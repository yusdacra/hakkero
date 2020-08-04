//! Implementation of `Log` trait.
use log::{Log, Metadata, Record};

pub static LOGGER: Logger = Logger;

/// Initializes the global logger in the `log` crate.
///
/// # Errors
/// Refer to `set_logger` function from the `log` crate.
pub fn init(level: log::LevelFilter) -> Result<(), log::SetLoggerError> {
    log::set_max_level(level);
    unsafe { log::set_logger_racy(&LOGGER) }
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let file_path = record.file().unwrap_or("<unknown file>");
            let line_number = record.line().unwrap_or(0);
            let log_level = record.level();
            let message = record.args();

            #[cfg(all(target_arch = "x86_64", feature = "log_vga"))]
            {
                use log::Level;
                use vga::colors::Color16;

                let color = match record.level() {
                    Level::Error => (Color16::Black, Color16::Red),
                    Level::Warn => (Color16::Yellow, Color16::Black),
                    Level::Info => (Color16::LightBlue, Color16::Black),
                    _ => (Color16::White, Color16::Black),
                };
                crate::println_colored!(
                    color,
                    "[{}:{}] [{}] {}",
                    file_path,
                    line_number,
                    log_level,
                    message,
                );
            }
            #[cfg(feature = "log_serial")]
            crate::serial_println!(
                "[{}:{}] [{}] {}",
                file_path,
                line_number,
                log_level,
                message,
            );
        }
    }

    fn flush(&self) {}
}
