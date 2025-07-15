use std::path::{Path, PathBuf};
use log::info;
use once_cell::sync::Lazy;

#[cfg(not(windows))]
pub fn get_appdata_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        std::env::current_dir().expect("cwd")
            .join("appdata")
    });
    return &*VALUE;
}

pub fn enter_char(c: char) {
    info!("mock: enter_char: {}", c);
}

pub fn enter_backspace() {
    info!("mock: backspace");
}

pub fn enter_enter() {
    info!("mock: enter");
}

pub fn enter_text(text: &str) -> bool {
    info!("mock: copy: {}", text);
    return true
}

pub fn copy_text(copy: &str) -> bool {
    info!("mock: copy: {}", copy);
    return true
}
