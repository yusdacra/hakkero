//! Implementation of `Log` trait.
use log::{LevelFilter, Log, Metadata, Record};

pub static LOGGER: Logger = Logger;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Trace;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

/// Initializes the global logger in the `log` crate.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub fn init() {
    log::set_max_level(LOG_LEVEL);
    if let Err(e) = log::set_logger(&LOGGER) {
        LOGGER.log(
            &log::Record::builder()
                .args(format_args!("Could not initialize the logger: {}", e))
                .level(log::Level::Error)
                .file(Some("src/misc/mod.rs"))
                .line(Some(11))
                .build(),
        );
    } else {
        log::info!("Successfully initialized the logger!");
    }
}

pub struct Logger;

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(debug_assertions)]
            let file_path = record.file().unwrap_or("<unknown file>");
            #[cfg(debug_assertions)]
            let line_number = record.line().unwrap_or(0);
            let log_level = record.level();
            let message = record.args();

            #[cfg(all(target_arch = "x86_64", feature = "log_vga"))]
            {
                #[cfg(debug_assertions)]
                crate::println!(
                    "[{}:{}] [{}] {}",
                    file_path,
                    line_number,
                    log_level,
                    message,
                );
                #[cfg(not(debug_assertions))]
                crate::println!("[{}] {}", log_level, message);
            }
            #[cfg(feature = "log_serial")]
            {
                #[cfg(debug_assertions)]
                crate::serial_println!(
                    "[{}:{}] [{}] {}",
                    file_path,
                    line_number,
                    log_level,
                    message,
                );
                #[cfg(not(debug_assertions))]
                crate::serial_println!("[{}] {}", log_level, message);
            }
        }
    }

    fn flush(&self) {}
}
