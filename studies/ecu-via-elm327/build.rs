fn main() {
    // Unlocks basic loads/stores in core
    // Unlocks CAS (Compare-and-Swap) in core
    //println!(
    //    "cargo:rustc-check-cfg=cfg(target_has_atomic_load_store, values(\"8\", \"16\", \"32\", \"ptr\"))"
    //);
    //println!("cargo:rustc-cfg=target_has_atomic_load_store=\"ptr\"");

    // Tells portable-atomic how to implement the missing CAS logic
    // Forces portable-atomic to use interrupt-disabling (the single-core "trick")
    println!("cargo:rustc-check-cfg=cfg(portable_atomic_unsafe_assume_single_core)");
    println!("cargo:rustc-cfg=target_has_atomic_load_store");

    // For embassy-executor you should also enable its portable-atomic feature
}
