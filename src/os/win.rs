use std::ffi::c_void;
use log::error;
use once_cell::sync::Lazy;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP, VK_LCONTROL, VK_LSHIFT};
use winsafe::co;
use winsafe::prelude::*;

pub fn get_appdata_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    return &*VALUE;
}

pub fn enter_char(c: char) {
    if '0' <= c && c <= '9' || 'a' <= c && c <= 'z' {
        // simple input.
        let c = c.to_ascii_uppercase() as u8;
        unsafe {
            keybd_event(c, 0, 0, 0);
            keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
        }
    } else if 'A' <= c && c <= 'Z' {
        // input with shift down
        let c = c as u8;
        unsafe {
            keybd_event(VK_LSHIFT as _, 0, 0, 0);
            keybd_event(c, 0, 0, 0);
            keybd_event(c, 0, KEYEVENTF_KEYUP, 0);
            keybd_event(VK_LSHIFT as _, 0, KEYEVENTF_KEYUP, 0);
        }
    } else {
        // fallback to copy & paste
        copy_text_and_enter_paste_shortcut(&c.to_string(), true);
    }
}

pub fn enter_backspace() {
    unsafe {
        // \x08: backspace
        keybd_event(b'\x08', 0, 0, 0);
        keybd_event(b'\x08', 0, KEYEVENTF_KEYUP, 0);
    }
}

pub fn enter_enter() {
    unsafe {
        // \x08: backspace
        keybd_event(b'\x08', 0, 0, 0);
        keybd_event(b'\x08', 0, KEYEVENTF_KEYUP, 0);
    }
}

pub(crate) fn copy_text_and_enter_paste_shortcut(copy: &str, paste: bool) -> bool {
    let _clipboard = match get_hwnd().OpenClipboard() {
        Ok(guard) => guard,
        Err(e) => {
            error!("could not possible to open clipboard: {e:?}");
            return false
        },
    };
    
    if let Err(e) = winsafe::EmptyClipboard() {
        error!("could not possible to clear clipboard: {e:?}");
        return false
    }

    let encoded = copy.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let mut shared_mem = match winsafe::HGLOBAL::GlobalAlloc(Some(co::GMEM::FIXED), encoded.len() * size_of::<u16>()) {
        Ok(guard) => guard,
        Err(e) => {
            error!("error in GlobalAlloc: {e:?}");
            return false
        },
    };

    let mut mem_region = match shared_mem.GlobalLock() {
        Ok(guard) => guard,
        Err(e) => {
            error!("error in GlobalLock: {e:?}");
            return false
        },
    };

    mem_region.as_mut_slice().copy_from_slice(bytemuck::cast_slice(&encoded));
    drop(mem_region);

    match unsafe { winsafe::SetClipboardData(co::CF::UNICODETEXT, shared_mem.leak().ptr() as *mut _) } {
        Ok(_) => {}
        Err(e) => {
            error!("error in SetClipboardData: {e:?}");
            return false
        },
    }

    if paste {
        unsafe {
            keybd_event(VK_LCONTROL as _, 0, Default::default(), 0);
            keybd_event(b'V', 0, Default::default(), 0);
            keybd_event(b'V', 0, KEYEVENTF_KEYUP, 0);
            keybd_event(VK_LCONTROL as _, 0, KEYEVENTF_KEYUP, 0);
        }
    }
    return true;
}

static CURRENT_HWND: std::sync::atomic::AtomicUsize = Default::default();

fn get_hwnd() -> winsafe::HWND {
    unsafe {
        winsafe::HWND::from_ptr(CURRENT_HWND.load(std::sync::atomic::Ordering::SeqCst) as *mut c_void)
    }
}

pub(crate) fn set_hwnd(hwnd: *mut c_void) {
    CURRENT_HWND.swap(hwnd as _, std::sync::atomic::Ordering::SeqCst);
}
