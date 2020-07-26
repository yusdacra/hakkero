pub fn wfe() {
    unsafe {
        llvm_asm!("wfe" :::: "volatile");
    }
}

pub fn sev() {
    unsafe {
        llvm_asm!("sev" :::: "volatile");
    }
}

pub fn wfi() {
    unsafe {
        llvm_asm!("wfi" :::: "volatile");
    }
}
