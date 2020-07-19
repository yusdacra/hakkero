#![no_std]
#![no_main]

use core::panic::PanicInfo;
use hakkero::{exit_qemu, serial_print, serial_println, QemuExitCode};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut sp = hakkero::serial::create_qemu_sp();

    should_fail(&mut sp);
    serial_println!(&mut sp, "[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail(sp: &mut hakkero::serial::SerialPort) {
    serial_print!(sp, "should_fail... ");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut sp = hakkero::serial::create_qemu_sp();
    serial_println!(&mut sp, "[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
