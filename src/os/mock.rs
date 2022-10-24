use std::path::{Path, PathBuf};
use once_cell::sync::Lazy;

#[cfg(not(windows))]
pub fn get_config_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        std::env::current_dir().expect("cwd")
    });
    return &*VALUE;
}

pub fn enter_char(c: char) {
    println!("mock: enter_char: {}", c);
}

pub fn enter_backspace() {
    println!("mock: backspace");
}

pub fn enter_enter() {
    println!("mock: enter");
}

pub fn copy_text_and_enter_paste_shortcut(copy: &str) {
    println!("mock: copy: {}", copy);
}
