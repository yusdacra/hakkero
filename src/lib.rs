//! The very powerful furnace OS.
#![no_std]
#![cfg_attr(test, no_main)]
#![feature(
    asm,
    decl_macro,
    custom_test_frameworks,
    abi_x86_interrupt,
    alloc_error_handler,
    naked_functions,
    const_fn,
    const_in_array_repeat_expressions,
    wake_trait,
    trait_alias,
    maybe_uninit_ref
)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(test::runner)]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::new_without_default, clippy::must_use_candidate)]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod memory;
pub mod misc;
pub mod task;
pub mod test;
