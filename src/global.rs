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

pub use crate::os::get_config_dir;
