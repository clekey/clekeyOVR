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

pub fn copy_text_and_enter_paste_shortcut(copy: &str, paste: bool) {
    info!("mock: copy: {}", copy);
}
