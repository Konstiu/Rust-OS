#[doc(hidden)]
pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::Dynamic);
    config
};

#[macro_export]
macro_rules! default_entry_point {
    ($path:path) => {
        bootloader_api::entry_point!($path, config = &$crate::entry_point::BOOTLOADER_CONFIG);
    };
}
