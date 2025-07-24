use log::*;
use once_cell::sync::Lazy;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;
use winsafe::{co, HwKbMouse, SendInput, KEYBDINPUT};

pub fn get_appdata_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    &VALUE
}

fn send_input<const N: usize>(keys: &[co::VK; N]) {
    //let mut inputs = [HwKbMouse::Kb(KEYBDINPUT::default()); N * 2];
    let mut inputs = vec![HwKbMouse::Kb(KEYBDINPUT::default()); N * 2];
    let mut index = 0;

    for &key in keys.iter() {
        inputs[index] = HwKbMouse::Kb(KEYBDINPUT {
            wVk: key,
            ..KEYBDINPUT::default()
        });
        index += 1;
    }

    for &key in keys.iter().rev() {
        inputs[index] = HwKbMouse::Kb(KEYBDINPUT {
            wVk: key,
            dwFlags: co::KEYEVENTF::KEYUP,
            ..KEYBDINPUT::default()
        });
        index += 1;
    }

    if let Err(e) = SendInput(&inputs) {
        error!("failed to send input: {}", e);
    }
}

pub fn enter_char(c: char) {
    if c.is_ascii_digit() || c.is_ascii_lowercase() {
        // simple input.
        let key = unsafe { co::VK::from_raw(c as u8 as u16) };
        send_input(&[key]);
    } else if c.is_ascii_uppercase() {
        // input with shift down
        let key = unsafe { co::VK::from_raw(c as u8 as u16) };
        send_input(&[co::VK::LSHIFT, key]);
    } else {
        // fallback to copy & paste
        enter_text(&c.to_string());
    }
}

pub fn enter_backspace() {
    send_input(&[co::VK::BACK]);
}

pub fn enter_enter() {
    send_input(&[co::VK::RETURN]);
}

pub fn enter_text(text: &str) -> bool {
    if let Err(e) = SendInput(
        &text
            .encode_utf16()
            .map(|c| {
                HwKbMouse::Kb(KEYBDINPUT {
                    wScan: c,
                    dwFlags: co::KEYEVENTF::UNICODE,
                    ..KEYBDINPUT::default()
                })
            })
            .collect::<Vec<_>>(),
    ) {
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
            }
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
            return false;
        }
    };

    if let Err(e) = clipboard.EmptyClipboard() {
        error!("could not possible to clear clipboard: {e:?}");
        return false;
    }

    let encoded = copy.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let clipboard_data =
        unsafe { std::slice::from_raw_parts(encoded.as_ptr() as *const u8, encoded.len() * 2) };

    match clipboard.SetClipboardData(co::CF::UNICODETEXT, clipboard_data) {
        Ok(_) => {}
        Err(e) => {
            error!("error in SetClipboardData: {e:?}");
            return false;
        }
    }

    true
}

static CURRENT_HWND: std::sync::atomic::AtomicPtr::<c_void> = std::sync::atomic::AtomicPtr::<c_void>::new(std::ptr::null_mut());

fn get_hwnd() -> winsafe::HWND {
    let hwnd_ptr = CURRENT_HWND.load(std::sync::atomic::Ordering::SeqCst);
    if hwnd_ptr.is_null() {
        let new_hwnd = {
            unsafe {
                winsafe::HWND::CreateWindowEx(
                    Default::default(),
                    winsafe::AtomStr::from_str("STATIC"),
                    None,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    None,
                    winsafe::IdMenu::None,
                    &winsafe::HINSTANCE::GetModuleHandle(None).unwrap(),
                    None
                ).unwrap()
            }
        };
        match CURRENT_HWND.compare_exchange(std::ptr::null_mut(), new_hwnd.ptr(), std::sync::atomic::Ordering::SeqCst, std::sync::atomic::Ordering::SeqCst) {
            Ok(_) => new_hwnd,
            Err(new) => {
                unsafe { winsafe::HWND::from_ptr(hwnd_ptr as _) }.DestroyWindow().ok();
                unsafe { winsafe::HWND::from_ptr(new) }
            }
        }
    } else {
       unsafe { winsafe::HWND::from_ptr(hwnd_ptr) }
    }
}
