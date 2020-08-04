# Hakkero OS
Learning project. Goal is to have a kernel which can run wasm code, that can run on x86_64, aarch64 and riscv64-gc.

## Prereqs
- rustup, cargo
- cargo-make (run `cargo install cargo-make`)
- QEMU

## Running
`cargo make` to lint, build and test `x86_64`.
`cargo make run` to build for `x86_64` and run using QEMU.
`cargo make -p aarch64 run` to build for `aarch64` and run on a `raspi3` machine using QEMU.
