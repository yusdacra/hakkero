pub fn wfe() {
    unsafe {
        asm!("wfe", options(nomem, nostack));
    }
}

pub fn sev() {
    unsafe {
        asm!("sev", options(nomem, nostack));
    }
}

pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}
