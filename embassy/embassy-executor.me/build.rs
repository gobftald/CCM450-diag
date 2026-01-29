fn main() {
    println!("cargo:rustc-check-cfg=cfg(cortex_m)");
}
