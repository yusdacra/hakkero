#[cfg(board = "virt")]
mod virt;
#[cfg(board = "virt")]
pub use virt::*;
#[cfg(board = "raspi3")]
mod raspi3;
#[cfg(board = "raspi3")]
pub use raspi3::*;
