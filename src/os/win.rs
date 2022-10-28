use std::ffi::c_int;
use std::mem::size_of;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::ptr::null;
use windows::Win32::Foundation::{GetLastError, HANDLE, HWND};
use windows::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalFree, GMEM_FIXED};
use windows::Win32::System::SystemServices::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{keybd_event, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_BACK, VK_LCONTROL, VK_LSHIFT};

pub fn get_config_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    return &*VALUE;
}

pub fn enter_char(c: char) {
    if '0' <= c && c <= '9' || 'a' <= c && c <= 'z' {
        // simple input.
        let c = c as u8 - b'a' + b'A';
        unsafe {
            keybd_event(c, 0, KEYBD_EVENT_FLAGS::default(), 0);
            keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
        }
    } else if 'A' <= c && c <= 'Z' {
        // input with shift down
        let c = c as u8;
        unsafe {
            keybd_event(VK_LSHIFT.0 as _, 0, KEYBD_EVENT_FLAGS::default(), 0);
            keybd_event(c, 0, KEYBD_EVENT_FLAGS::default(), 0);
            keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
            keybd_event(VK_LSHIFT.0 as _, 0, KEYEVENTF_KEYUP, 0);
        }
    } else {
        // fallback to copy & paste
        copy_text_and_enter_paste_shortcut(&c.to_string());
    }
}

pub fn enter_backspace() {
    unsafe {
        // \x08: backspace
        keybd_event(b'\x08', 0, KEYBD_EVENT_FLAGS::default(), 0);
        keybd_event(b'\x08', 0, KEYEVENTF_KEYUP, 0);
    }
}

pub fn enter_enter() {
    unsafe {
        // \x08: backspace
        keybd_event(b'\r', 0, KEYBD_EVENT_FLAGS::default(), 0);
        keybd_event(b'\r', 0, KEYEVENTF_KEYUP, 0);
    }
}

pub(crate) fn copy_text_and_enter_paste_shortcut(copy: &str) {
    // copy string
    unsafe {
        if !OpenClipboard(HWND::default()).as_bool() {
            eprintln!("could not possible to open clipboard: {:?}", std::io::Error::last_os_error());
            return
        }

        if !EmptyClipboard().as_bool() {
            eprintln!("could not possible to clear clipboard: {:?}", std::io::Error::last_os_error());
            return
        }
        
        let encoded = copy.encode_utf16().chain([0]).collect::<Vec<u16>>();

        let allocated = GlobalAlloc(GMEM_FIXED, encoded.len() * size_of::<u16>());
        if allocated == 0 {
            eprintln!("error in GlobalAlloc: {:?}", std::io::Error::last_os_error());
            return
        }
        let allocated = HANDLE(allocated);
        match SetClipboardData(CF_UNICODETEXT.0, allocated) {
            Ok(_) => {},
            Err(e) => {
                eprintln!("error in SetClipboardData: {:?}", std::io::Error::last_os_error());
                GlobalFree(allocated.0);
                return;
            }
        }
        CloseClipboard();

        keybd_event(VK_LCONTROL.0 as _, 0, Default::default(), 0);
        keybd_event(b'V', 0, Default::default(), 0);
        keybd_event(b'V', 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LCONTROL.0 as _, 0, KEYEVENTF_KEYUP, 0);
    }
}
