use log::{Log, Metadata, Record};

static LOGGER: Logger = Logger;

pub struct Logger;

impl Logger {
    pub fn init(level: log::LevelFilter) {
        log::set_logger(&LOGGER).expect("Could not init logger");
        log::set_max_level(level);
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(target_arch = "x86_64")]
            {
                #[cfg(feature = "log_vga")]
                {
                    use log::Level;
                    use vga::colors::Color16;

                    let color = match record.level() {
                        Level::Error => (Color16::Black, Color16::Red),
                        Level::Warn => (Color16::Yellow, Color16::Black),
                        Level::Info => (Color16::LightBlue, Color16::Black),
                        _ => (Color16::White, Color16::Black),
                    };
                    crate::println_colored!(color, "[{:5}] {}", record.level(), record.args());
                }
                #[cfg(feature = "log_serial")]
                {
                    crate::serial_println!("[{:5}] {}", record.level(), record.args());
                }
            }
            #[cfg(target_arch = "aarch64")]
            {
                #[cfg(feature = "log_serial")]
                {
                    crate::console_println!("[{:5}] {}", record.level(), record.args());
                }
            }
        }
    }

    fn flush(&self) {}
}
