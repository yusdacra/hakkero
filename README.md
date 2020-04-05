# Hakkero OS
My implementation of https://os.phil-opp.com/. The goal is to learn, but a more long term goal might be using it personally. Any help is welcome as I'm not experienced.

## Prereqs
- Nightly Rust compiler (with rust-src and llvm-tools-preview installed)
- cargo-xbuild ~ for cross-compiling
- bootimage ~ to create a bootable image
- QEMU ~ to easily test the the OS

## Building
0. Make sure to override the project to use nightly toolchain
1. `cargo-xrun` to launch the OS in QEMU
