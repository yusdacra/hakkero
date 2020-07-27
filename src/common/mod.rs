//! Misc common code used in many places.
pub mod logger;
pub mod once;

pub use logger::Logger;
pub use once::Once;

pub fn init() {
    Logger::init(log::LevelFilter::Trace);
}