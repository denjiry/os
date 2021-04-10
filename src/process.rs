#[repr(C)]
struct Context {
    // offset 0x00
    cr3: u64,
    rip: u64,
    rflags: u64,
    reserved1: u64,
    // offset 0x20
    cs: u64,
    ss: u64,
    fs: u64,
    gs: u64,
    // offset 0x40
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rdi: u64,
    rsi: u64,
    rsp: u64,
    rbp: u64,
    // offset 0x80
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
    // offset 0xc0
    fxsave_area: [u8; 512],
}
