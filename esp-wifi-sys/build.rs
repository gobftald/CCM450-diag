use std::{
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;

fn main() -> Result<()> {
    // Put the linker script somewhere the linker can find it
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    #[cfg(feature = "esp32c3")]
    {
        copy_libraries(&out)?;
    }

    println!("cargo:rustc-link-search={}", out.display());

    Ok(())
}

fn copy_file(out: &PathBuf, from: &str, to: &str) -> Result<()> {
    let mut file = File::create(out.join(to))?;
    file.write_all(&fs::read(from)?)?;

    Ok(())
}

#[cfg(feature = "esp32c3")]
fn copy_libraries(out: &PathBuf) -> Result<()> {
    copy_file(out, "libs/esp32c3/libcore.a", "libcore.a")?;
    copy_file(out, "libs/esp32c3/libmesh.a", "libmesh.a")?;
    copy_file(out, "libs/esp32c3/libnet80211.a", "libnet80211.a")?;
    copy_file(out, "libs/esp32c3/libphy.a", "libphy.a")?;
    copy_file(out, "libs/esp32c3/libpp.a", "libpp.a")?;
    copy_file(
        out,
        "libs/esp32c3/libwpa_supplicant.a",
        "libwpa_supplicant.a",
    )?;
    copy_file(out, "libs/esp32c3/libprintf.a", "libprintf.a")?;

    println!("cargo:rustc-link-lib={}", "core");
    println!("cargo:rustc-link-lib={}", "mesh");
    println!("cargo:rustc-link-lib={}", "net80211");
    println!("cargo:rustc-link-lib={}", "phy");
    println!("cargo:rustc-link-lib={}", "pp");
    println!("cargo:rustc-link-lib={}", "wpa_supplicant");
    println!("cargo:rustc-link-lib={}", "printf");

    Ok(())
}
