#[cfg(not(all(
    target_pointer_width = "32",
    any(
        target_arch = "aarch64",
        target_arch = "amdgpu",
        target_arch = "arm64ec",
        target_arch = "bpf",
        target_arch = "loongarch64",
        target_arch = "mips64",
        target_arch = "mips64r6",
        target_arch = "nvptx64",
        target_arch = "powerpc64",
        target_arch = "riscv64",
        target_arch = "s390x",
        target_arch = "sparc64",
        target_arch = "wasm64",
        target_arch = "x86_64",
    ),
)))]
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
macro_rules! ptr_reg {
    ($ptr:ident) => {{
        let _: *const _ = $ptr; // ensure $ptr is a pointer (*mut _ or *const _)
        $ptr // cast is unnecessary here.
    }};
}
