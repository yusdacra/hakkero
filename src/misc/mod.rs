//! Misc common code used in many places.
pub mod logger;
pub mod once;

pub use once::Once;

#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub fn init() {
    if let Err(e) = logger::init(log::LevelFilter::Trace) {
        use log::Log;

        logger::LOGGER.log(
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
