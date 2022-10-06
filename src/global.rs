use core::panicking::panic;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

pub fn get_exe_dir() -> PathBuf {
    std::env::current_exe()
        .expect("failed to get current exe path")
        .parent()
        .expect("current exe is not in a folder")
        .to_owned()
}

pub fn get_resources_dir() -> PathBuf {
    get_exe_dir().join("resources")
}

#[cfg(windows)]
pub fn get_config_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    return &*VALUE;
}

#[cfg(not(windows))]
pub fn get_config_dir() -> &'static Path {
    panic!("not supported")
}
