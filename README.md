# Hakkero OS
My implementation of https://os.phil-opp.com/. Some extras here and there, and (probably) it will advance quicker than the blog. The goal is to learn, but a more long term goal might be using it personally. Any help is welcome as I'm not experienced.

## Prereqs
- Nightly rust compiler (with rust-src and llvm-preview-whatever installed)
- Cargo-xbuild ~ for cross-compiling
- Bootimage ~ to create a bootable image
- QEMU ~ for running the OS virtually

## Building
0. make sure to override the project to use nightly toolchain
1. `cargo-xrun` to launch the OS in QEMU