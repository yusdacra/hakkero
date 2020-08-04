#![allow(clippy::doc_markdown, unused_macros)]

#[allow(clippy::cast_possible_truncation, clippy::inline_always)]
#[inline(always)]
pub fn core_id() -> u8 {
    (mpidr_el1::read() & mpidr_el1::CORE_ID_MASK) as u8
}

macro_rules! read {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn read() -> $size {
            let value;
            unsafe {
                asm!(concat!("mrs {}, ", $name), out(reg) value);
            }
            value
        }
    };
}

macro_rules! write {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn write(value: $size) {
            unsafe {
                asm!(concat!("msr ", $name, ", {}"), in(reg) value);
            }
        }
    };
}

macro_rules! set {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn set(fields: $size) {
            write(read() | fields);
        }
    };
}

macro_rules! get {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn get(fields: $size) -> $size {
            read() & fields
        }
    };
}

macro_rules! clear {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn clear(fields: $size) {
            set(!fields);
        }
    };
}

macro_rules! is_all_set {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn is_all_set(fields: $size) -> bool {
            read() & fields == fields
        }
    };
}

macro_rules! is_any_set {
    ($size:ty, $name:tt) => {
        #[allow(clippy::inline_always)]
        #[inline(always)]
        pub fn is_any_set(fields: $size) -> bool {
            read() & fields != 0
        }
    };
}

/// Multiprocessor Affinity register.
pub mod mpidr_el1 {
    pub const CORE_ID_MASK: u64 = 0b11;

    read!(u64, "mpidr_el1");
}
pub mod elr_el2 {
    write!(u64, "elr_el2");
}
pub mod cnthctl_el2 {
    /// Traps EL0 and EL1 accesses to the EL1 physical counter register to EL2 when EL2 is enabled
    /// in the current Security state.
    ///
    /// 0b0 - Accesses to the CNTPCT_EL0 are trapped to EL2
    /// 0b1 - No instructions to be trapped
    pub const EL1PCTEN: u64 = 1;

    /// Traps EL0 and EL1 accesses to the EL1 physical timer registers to EL2 when EL2 is enabled
    /// in the current Security state
    ///
    /// 0b0 - Access to CNTP_CTL_EL0, CNTP_CVAL_EL0, and CNTP_TVAL_EL0 are trapped to EL2
    /// 0b1 - No instructions to be trapped
    pub const EL1PCEN: u64 = 1 << 1;

    set!(u64, "cnthctl_el2");
    write!(u64, "cnthctl_el2");
    read!(u64, "cnthctl_el2");
}
pub mod cntvoff_el2 {
    write!(u64, "cntvoff_el2");
}
pub mod hcr_el2 {
    /// Execution state control for lower Exception levels
    ///
    /// 0b0 - Lower levels are all AArch32
    /// 0b1 - The Execution state for EL1 is AArch64
    pub const RW: u64 = 1 << 31;

    /// Set/Way Invalidation Override
    ///
    /// 0b0 - No effect
    /// 0b1 - Data cache invalidate by set/way instructions
    pub const SWIO: u64 = 1 << 1;

    set!(u64, "hcr_el2");
    write!(u64, "hcr_el2");
    read!(u64, "hcr_el2");
}
pub mod spsr_el2 {
    /// Debug exception mask
    pub const D: u64 = 1 << 9;
    /// SError exception mask
    pub const A: u64 = 1 << 8;
    /// IRQ exception mask
    pub const I: u64 = 1 << 7;
    /// FIQ exception mask
    pub const F: u64 = 1 << 6;
    pub const EL1H: u64 = 0b0101;

    write!(u64, "spsr_el2");
}
pub mod spsel {
    /// Use SP_ELX for Exception level ELX
    /// SP_EL0 if unset
    pub const SP_ELX: u64 = 1;

    write!(u64, "spsel");
}
pub mod currentel {
    pub const EL3_VALUE: u64 = 0b11 << 2;
    pub const EL2_VALUE: u64 = 0b10 << 2;
    pub const EL1_VALUE: u64 = 0b01 << 2;
    pub const EL0_VALUE: u64 = 0b00 << 2;
    pub const EL_MASK: u64 = 0b11 << 2;

    read!(u64, "currentel");
}
