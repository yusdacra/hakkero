//! Stuff needed for testing.
use crate::serial_println;
use core::panic::PanicInfo;

pub fn runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[allow(unused_variables)]
pub fn panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    #[cfg(target_arch = "x86_64")]
    {
        use x86_64::instructions::port::PortWriteOnly;

        unsafe {
            let mut port = PortWriteOnly::new(0xf4);
            port.write(exit_code as u32);
        }
    }
    // TODO: Actually implement this
    #[cfg(target_arch = "aarch64")]
    {
        let _holder = exit_code;
        unsafe {
            asm!("ldr x0, =0x84000008");
            asm!("hvc #0");
        }
    }
}

#[cfg(test)]
crate::arch::entry_point!(kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn kernel_main() -> ! {
    crate::test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic_handler(info)
}
