use std::ffi::c_void;
use log::*;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP, VK_LSHIFT};
use winsafe::{co, HwKbMouse, SendInput, KEYBDINPUT};

pub fn get_appdata_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    &*VALUE
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
        enter_text(&c.to_string());
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
        // \r: enter
        keybd_event(b'\r', 0, 0, 0);
        keybd_event(b'\r', 0, KEYEVENTF_KEYUP, 0);
    }
}

pub fn enter_text(text: &str) -> bool {
    if let Err(e) = SendInput(&text.encode_utf16().map(|c| {
        HwKbMouse::Kb(KEYBDINPUT {
            wScan: c,
            dwFlags: co::KEYEVENTF::UNICODE,
            ..KEYBDINPUT::default()
        })
    }).collect::<Vec<_>>()) {
        error!("failed to send text: {}", e);
        return false;
    }
    true
}

fn open_clipboard(hwnd: &winsafe::HWND) -> winsafe::SysResult<winsafe::guard::CloseClipboardGuard> {
    for i in 0..9 {
        match hwnd.OpenClipboard() {
            Ok(guard) => return Ok(guard),
            Err(e) => {
                info!("open failure #{i}: {e:?}");
            },
        };
        sleep(Duration::from_millis(100));
    }
    hwnd.OpenClipboard()
}

pub(crate) fn copy_text(copy: &str) -> bool {
    let hwnd = get_hwnd();
    let clipboard = match open_clipboard(&hwnd) {
        Ok(guard) => guard,
        Err(e) => {
            error!("could not possible to open clipboard: {e:?}");
            return false
        },
    };
    
    if let Err(e) = clipboard.EmptyClipboard() {
        error!("could not possible to clear clipboard: {e:?}");
        return false
    }

    let encoded = copy.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let clipboard_data = unsafe { std::slice::from_raw_parts(encoded.as_ptr() as *const u8, encoded.len() * 2) };

    match clipboard.SetClipboardData(co::CF::UNICODETEXT, clipboard_data) {
        Ok(_) => {}
        Err(e) => {
            error!("error in SetClipboardData: {e:?}");
            return false
        },
    }

    true
}

static CURRENT_HWND: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn get_hwnd() -> winsafe::HWND {
    unsafe {
        winsafe::HWND::from_ptr(CURRENT_HWND.load(std::sync::atomic::Ordering::SeqCst) as *mut c_void)
    }
}

pub(crate) fn set_hwnd(hwnd: *mut c_void) {
    CURRENT_HWND.swap(hwnd as _, std::sync::atomic::Ordering::SeqCst);
}
