#!/usr/bin/env bash
# TODO: Probably use a makefile or something

AARCH64_DEF_BSP="rpi"

if [[ "$1" = "-v" ]]; then
    VERBOSE="--verbose"
    COM=$2
    ARCH=$3
    BSP=$4
else
    VERBOSE=""
    COM=$1
    ARCH=$2
    BSP=$3
fi

if [[ "$BSP" = "" ]]; then
    BSP=$AARCH64_DEF_BSP
fi

function x86_64_cmd () {
    cargo $1 $VERBOSE --target targets/x86_64-hakkero.json || exit 1
}

function aarch64_cmd () {
    if [[ ! -r "src/arch/aarch64/device/$2/link.ld" ]]; then
        echo "BSP $2 isn't supported for aarch64"
        exit 1
    fi
    RUSTFLAGS="-C link-arg=-Tsrc/arch/aarch64/device/$2/link.ld" cargo $1 $VERBOSE --target targets/aarch64-hakkero.json || exit 1
}

function aarch64_run() {
    KERNEL="target/hakkero.img"
    aarch64_cmd xbuild $1
    rust-objcopy --strip-all -O binary target/aarch64-hakkero/debug/hakkero $KERNEL
    qemu-system-aarch64 -kernel $KERNEL -serial stdio -display none -machine raspi3
}

case "$COM" in
    "run")
        case "$ARCH" in
            "x86_64")
                x86_64_cmd xrun
                ;;
            "aarch64")
                aarch64_run $BSP
                ;;
            "all")
                x86_64_cmd xrun
                aarch64_run $BSP
                ;;
            *)
                echo "Arch $ARCH not supported for this command"
                ;;
        esac
        ;;
    "test")
        case "$ARCH" in
            "x86_64")
                x86_64_cmd xtest
                ;;
            "aarch64")
                echo "aarch64 does not support testing (yet)"
                ;;
            "all")
                x86_64_cmd xtest
                echo "aarch64 does not support testing (yet)"
                ;;
            *)
                echo "Arch $ARCH not supported for this command"
                ;;
        esac
        ;;
    "check")
        case "$ARCH" in
            "x86_64")
                x86_64_cmd xclippy
                ;;
            "aarch64")
                aarch64_cmd xclippy $BSP
                ;;
            "all")
                x86_64_cmd xclippy
                aarch64_cmd xclippy $BSP
                ;;
            *)
                echo "Arch $ARCH not supported for this command"
                ;;
        esac
        ;;
    "doc")
        case "$ARCH" in
            "x86_64")
                x86_64_cmd xdoc
                ;;
            "aarch64")
                aarch64_cmd xdoc $BSP
                ;;
            *)
                echo "Arch $ARCH not supported for this command"
                ;;
        esac
        ;;
    "help")
        echo "Usage: build.sh [FLAG] <command> <arch>"
        echo "Commands: run, build, check"
        echo "Supported architectures: x86_64, aarch64, all (runs command for all archs)"
        echo "Supported aarch64 BSPs: rpi"
        echo "Use -v as FLAG to get verbose output."
        ;;
    *)
        echo "Command $COM not found"
        ;;
esac
