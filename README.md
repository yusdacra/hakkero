# Hakkero OS
My implementation of https://os.phil-opp.com/. Currently implements up to post 12. Contains some extra stuff too.

## Prereqs
- `rust-src` and `llvm-tools-preview` components
- cargo-xbuild ~ for cross-compiling
- bootimage ~ to create a bootable image
- QEMU ~ to work on and test the OS

## Building
- `cargo-xrun` to launch the OS in QEMU
- `cargo-xtest` to run the tests
