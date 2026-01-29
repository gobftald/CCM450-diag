#![no_std]

/// ESP-IDF compatible application descriptor
///
/// This gets populated by the [esp_app_desc] macro.
// 131
#[repr(C)]
pub struct EspAppDesc {
    /// Magic word ESP_APP_DESC_MAGIC_WORD
    magic_word: u32,
    /// Secure version
    secure_version: u32,
    /// Reserved
    reserv1: [u32; 2],
    /// Application version
    version: [core::ffi::c_char; 32],
    /// Project name
    project_name: [core::ffi::c_char; 32],
    /// Compile time
    time: [core::ffi::c_char; 16],
    /// Compile date
    date: [core::ffi::c_char; 16],
    /// Version IDF
    idf_ver: [core::ffi::c_char; 32],
    /// sha256 of elf file
    app_elf_sha256: [u8; 32],
    /// Minimal eFuse block revision supported by image, in format: major * 100
    /// + minor
    min_efuse_blk_rev_full: u16,
    /// Maximal eFuse block revision supported by image, in format: major * 100
    /// + minor
    max_efuse_blk_rev_full: u16,
    /// MMU page size in log base 2 format
    mmu_page_size: u8,
    /// Reserved
    reserv3: [u8; 3],
    /// Reserved
    reserv2: [u32; 18],
}

// 165
impl EspAppDesc {
    pub const fn new_internal(
        version: &str,
        project_name: &str,
        build_time: &str,
        build_date: &str,
        idf_ver: &str,
        min_efuse_blk_rev_full: u16,
        max_efuse_blk_rev_full: u16,
        mmu_page_size: u32,
    ) -> Self {
        Self {
            magic_word: ESP_APP_DESC_MAGIC_WORD,
            secure_version: 0,
            reserv1: [0; 2],
            version: str_to_cstr_array(version),
            project_name: str_to_cstr_array(project_name),
            time: str_to_cstr_array(build_time),
            date: str_to_cstr_array(build_date),
            idf_ver: str_to_cstr_array(idf_ver),
            app_elf_sha256: [0; 32],
            min_efuse_blk_rev_full,
            max_efuse_blk_rev_full,
            mmu_page_size: (mmu_page_size.ilog2()) as u8,
            reserv3: [0; 3],
            reserv2: [0; 18],
        }
    }
}

// 317
const ESP_APP_DESC_MAGIC_WORD: u32 = 0xABCD5432;

// 319
const fn str_to_cstr_array<const C: usize>(s: &str) -> [::core::ffi::c_char; C] {
    let bytes = s.as_bytes();
    let mut ret: [::core::ffi::c_char; C] = [0; C];
    let mut i = 0;
    loop {
        ret[i] = bytes[i] as _;
        i += 1;
        if i >= bytes.len() || i >= C {
            break;
        }
    }
    ret
}

/// Build time
// 334
pub const BUILD_TIME: &str = env!("ESP_BOOTLOADER_BUILD_TIME");

/// Build date
// 337
pub const BUILD_DATE: &str = env!("ESP_BOOTLOADER_BUILD_DATE");

/// MMU page size in bytes
// 340
pub const MMU_PAGE_SIZE: u32 = {
    let mmu_page_size =
        esp_config::esp_config_str!("ESP_BOOTLOADER_ESP_IDF_CONFIG_MMU_PAGE_SIZE").as_bytes();
    match mmu_page_size {
        b"8k" => 8 * 1024,
        b"16k" => 16 * 1024,
        b"32k" => 32 * 1024,
        b"64k" => 64 * 1024,
        _ => 64 * 1024,
    }
};

/// The (pretended) ESP-IDF version
// 353
pub const ESP_IDF_COMPATIBLE_VERSION: &str =
    esp_config::esp_config_str!("ESP_BOOTLOADER_ESP_IDF_CONFIG_ESP_IDF_VERSION");

// 360
/// This macro populates the application descriptor (see [EspAppDesc]) which is
/// available as a static named `ESP_APP_DESC`
///
/// In most cases you can just use the no-arguments version of this macro.
#[macro_export]
macro_rules! esp_app_desc {
    () => {
        $crate::esp_app_desc!(
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_NAME"),
            $crate::BUILD_TIME,
            $crate::BUILD_DATE,
            $crate::ESP_IDF_COMPATIBLE_VERSION,
            $crate::MMU_PAGE_SIZE,
            0,
            u16::MAX
        );
    };

    (
     $version: expr,
     $project_name: expr,
     $build_time: expr,
     $build_date: expr,
     $idf_ver: expr,
     $mmu_page_size: expr,
     $min_efuse_blk_rev_full: expr,
     $max_efuse_blk_rev_full: expr
    ) => {
        #[unsafe(export_name = "esp_app_desc")]
        #[unsafe(link_section = ".rodata_desc.appdesc")]
        /// Application metadata descriptor.
        pub static ESP_APP_DESC: $crate::EspAppDesc = $crate::EspAppDesc::new_internal(
            $version,
            $project_name,
            $build_time,
            $build_date,
            $idf_ver,
            $min_efuse_blk_rev_full,
            $max_efuse_blk_rev_full,
            $mmu_page_size,
        );
    };
}
