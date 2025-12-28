#[path = "version.rs"]
mod version;
use self::version::{rustc_version, Version};

#[path = "src/gen/build.rs"]
mod generated;

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/gen/build.rs");
    println!("cargo:rerun-if-changed=version.rs");

    #[cfg(feature = "unsafe-assume-single-core")]
    println!("cargo:rustc-cfg=portable_atomic_unsafe_assume_single_core");

    let target = &*env::var("TARGET").expect("TARGET not set");
    eprintln!("target = {}", target); // riscv32imc-unknown-none-elf
    let target_arch = &*env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH not set");
    eprintln!("target_arch = {}", target_arch); // riscv32
    let target_os = &*env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS not set");
    eprintln!("target_os = {}", target_os); // none

    let version = match rustc_version() {
        Some(version) => version,
        None => {
            if env::var_os("PORTABLE_ATOMIC_DENY_WARNINGS").is_some() {
                panic!("unable to determine rustc version")
            }
            println!(
                "cargo:warning={}: unable to determine rustc version; assuming latest stable rustc (1.{})",
                env!("CARGO_PKG_NAME"),
                Version::LATEST.minor
            );
            Version::LATEST
        }
    }; // Version { minor: 86, nightly: false, commit_date: Date { year: 0, month: 0, day: 0 }, llvm: 19 }

    if version.minor >= 80 {
        println!(
            /*
            cargo:rustc-check-cfg=
                cfg(
                    target_feature,
                    values(
                        "lsfe",
                        "fast-serialization",
                        "load-store-on-cond",
                        "distinct-ops"
                    )
                )
            */
            r#"cargo:rustc-check-cfg=cfg(target_feature,values("lsfe","fast-serialization","load-store-on-cond","distinct-ops"))"#
        );
        println!(
            /*
            cargo:rustc-check-cfg=
                cfg(
                    portable_atomic_disable_fiq,
                    portable_atomic_force_amo,
                    portable_atomic_ll_sc_rmw,
                    portable_atomic_atomic_intrinsics,
                    portable_atomic_no_asm,
                    portable_atomic_no_asm_maybe_uninit,
                    portable_atomic_no_atomic_64,
                    portable_atomic_no_atomic_cas,
                    portable_atomic_no_atomic_load_store,
                    portable_atomic_no_atomic_min_max,
                    portable_atomic_no_cfg_target_has_atomic,
                    portable_atomic_no_cmpxchg16b_intrinsic,
                    portable_atomic_no_cmpxchg16b_target_feature,
                    portable_atomic_no_const_mut_refs,
                    portable_atomic_no_const_raw_ptr_deref,
                    portable_atomic_no_const_transmute,
                    portable_atomic_no_core_unwind_safe,
                    portable_atomic_no_diagnostic_namespace,
                    portable_atomic_no_strict_provenance,
                    portable_atomic_no_stronger_failure_ordering,
                    portable_atomic_no_track_caller,
                    portable_atomic_no_unsafe_op_in_unsafe_fn,
                    portable_atomic_pre_llvm_15,
                    portable_atomic_pre_llvm_16,
                    portable_atomic_pre_llvm_18,
                    portable_atomic_pre_llvm_20,
                    portable_atomic_s_mode,
                    portable_atomic_sanitize_thread,
                    portable_atomic_target_feature,

                    portable_atomic_unsafe_assume_single_core,
                    
                    portable_atomic_unstable_asm,
                    portable_atomic_unstable_asm_experimental_arch,
                    portable_atomic_unstable_cfg_target_has_atomic,
                    portable_atomic_unstable_isa_attribute
                )
            */
            "cargo:rustc-check-cfg=cfg(portable_atomic_disable_fiq,portable_atomic_force_amo,portable_atomic_ll_sc_rmw,portable_atomic_atomic_intrinsics,portable_atomic_no_asm,portable_atomic_no_asm_maybe_uninit,portable_atomic_no_atomic_64,portable_atomic_no_atomic_cas,portable_atomic_no_atomic_load_store,portable_atomic_no_atomic_min_max,portable_atomic_no_cfg_target_has_atomic,portable_atomic_no_cmpxchg16b_intrinsic,portable_atomic_no_cmpxchg16b_target_feature,portable_atomic_no_const_mut_refs,portable_atomic_no_const_raw_ptr_deref,portable_atomic_no_const_transmute,portable_atomic_no_core_unwind_safe,portable_atomic_no_diagnostic_namespace,portable_atomic_no_strict_provenance,portable_atomic_no_stronger_failure_ordering,portable_atomic_no_track_caller,portable_atomic_no_unsafe_op_in_unsafe_fn,portable_atomic_pre_llvm_15,portable_atomic_pre_llvm_16,portable_atomic_pre_llvm_18,portable_atomic_pre_llvm_20,portable_atomic_s_mode,portable_atomic_sanitize_thread,portable_atomic_target_feature,portable_atomic_unsafe_assume_single_core,portable_atomic_unstable_asm,portable_atomic_unstable_asm_experimental_arch,portable_atomic_unstable_cfg_target_has_atomic,portable_atomic_unstable_isa_attribute)"
        );
        println!(
            /*
            cargo:rustc-check-cfg=
                cfg(
                    portable_atomic_target_feature,
                    values(
                        "cmpxchg16b",
                        "distinct-ops",
                        "fast-serialization",
                        "load-store-on-cond",
                        "lse",
                        "lse128",
                        "lse2",
                        "lsfe",
                        "mclass",
                        "miscellaneous-extensions-3",
                        "quadword-atomics",
                        "rcpc3",
                        "v6",
                        "zaamo",
                        "zabha",
                        "zacas"
                    )
                )
            */
            r#"cargo:rustc-check-cfg=cfg(portable_atomic_target_feature,values("cmpxchg16b","distinct-ops","fast-serialization","load-store-on-cond","lse","lse128","lse2","lsfe","mclass","miscellaneous-extensions-3","quadword-atomics","rcpc3","v6","zaamo","zabha","zacas"))"#
        );
    }

    if version.nightly
        && version.probe(64, 2022, 6, 29)
        && !version.probe(89, 2025, 5, 30)
        && (target_arch != "powerpc64" || version.llvm >= 15)
    {
        println!("cargo:rustc-cfg=portable_atomic_atomic_intrinsics");
    }

    let no_asm = !version.probe(59, 2021, 12, 15); // false
    if no_asm {
        if version.nightly
            // this will be true
            && version.probe(46, 2020, 6, 20)
            // this will be true
            && ((target_arch != "x86" && target_arch != "x86_64") || version.llvm >= 10)
            // this will be true
            && is_allowed_feature("asm")
        {
            println!("cargo:rustc-cfg=portable_atomic_unstable_asm");
        }
        println!("cargo:rustc-cfg=portable_atomic_no_asm");
    } else {
        match target_arch {
            "arm64ec" | "s390x" => {}
            "powerpc64" => {}
            _ => {}
        }
    }

    if version.llvm < 20 {
        println!("cargo:rustc-cfg=portable_atomic_pre_llvm_20"); // here we are
        if version.llvm < 18 {
            println!("cargo:rustc-cfg=portable_atomic_pre_llvm_18");
            if version.llvm < 16 {
                println!("cargo:rustc-cfg=portable_atomic_pre_llvm_16");
                if version.llvm < 15 {
                    println!("cargo:rustc-cfg=portable_atomic_pre_llvm_15");
                }
            }
        }
    }

    match target_arch {
        "riscv32" | "riscv64" => {
            let mut _zaamo = false;
            // this will be true
            if !version.probe(87, 2025, 2, 25) || needs_target_feature_fallback(&version, None) {
                // this will be false
                _zaamo |= target_feature_fallback("zacas", false);
            }
        }
        _ => {}
    }
}

fn needs_target_feature_fallback(version: &Version, stable: Option<u32>) -> bool {
    match stable {
        // In these cases, cfg(target_feature = "...") would work, so skip emitting our own fallback target_feature cfg.
        _ if version.nightly => false,
        Some(stabilized) if version.minor >= stabilized => false,
        _ => true,
    }
}

fn target_feature_fallback(name: &str, mut has_target_feature: bool) -> bool {
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = strip_prefix(flag, "-C").unwrap_or(flag);
            if let Some(flag) = strip_prefix(flag, "target-feature=") {
                for s in flag.split(',') {
                    // TODO: Handles cases where a specific target feature
                    // implicitly enables another target feature.
                    match (s.as_bytes().first(), s.as_bytes().get(1..)) {
                        (Some(b'+'), Some(f)) if f == name.as_bytes() => has_target_feature = true,
                        (Some(b'-'), Some(f)) if f == name.as_bytes() => has_target_feature = false,
                        _ => {}
                    }
                }
            }
        }
    }
    if has_target_feature {
        println!(
            "cargo:rustc-cfg=portable_atomic_target_feature=\"{}\"",
            name
        );
    }
    has_target_feature
}

fn is_allowed_feature(name: &str) -> bool {
    // https://github.com/dtolnay/thiserror/pull/248
    if env::var_os("RUSTC_STAGE").is_some() {
        return false;
    }

    // allowed by default
    let mut allowed = true;
    if let Some(rustflags) = env::var_os("CARGO_ENCODED_RUSTFLAGS") {
        for mut flag in rustflags.to_string_lossy().split('\x1f') {
            flag = strip_prefix(flag, "-Z").unwrap_or(flag);
            if let Some(flag) = strip_prefix(flag, "allow-features=") {
                // If it is specified multiple times, the last value will be preferred.
                allowed = flag.split(',').any(|allowed| allowed == name);
            }
        }
    }
    allowed
}

#[must_use]
fn strip_prefix<'a>(s: &'a str, pat: &str) -> Option<&'a str> {
    if s.starts_with(pat) {
        Some(&s[pat.len()..])
    } else {
        None
    }
}
