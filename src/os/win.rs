use log::error;
use once_cell::sync::Lazy;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use winsafe::{co, HwKbMouse, KEYBDINPUT};
use winsafe::prelude::*;

pub fn get_appdata_dir() -> &'static Path {
    static VALUE: Lazy<PathBuf> = Lazy::new(|| {
        PathBuf::from(std::env::var_os("APPDATA").expect("no APPDATA found")).join("clekey_ovr")
    });
    return &*VALUE;
}

pub fn enter_char(c: char) {
    let (vk, shift) = char_to_key_code(c);
    if vk == co::VK::NoValue {
        // fallback to copy & paste
        copy_text_and_enter_paste_shortcut(&c.to_string());
    } else {
        if shift {
            input_two_key(co::VK::LSHIFT, vk);
        }else{
            input_one_key(vk);
        }
    }
}

pub fn enter_backspace() {
    input_one_key(co::VK::BACK);
}

pub fn enter_enter() {
    input_one_key(co::VK::RETURN);
}

pub(crate) fn copy_text_and_enter_paste_shortcut(copy: &str) {
    let _clipboard = match winsafe::HWND::NULL.OpenClipboard() {
        Ok(guard) => guard,
        Err(e) => return error!("could not possible to open clipboard: {e:?}"),
    };
    
    if let Err(e) = winsafe::EmptyClipboard() {
        return error!("could not possible to clear clipboard: {e:?}");
    }

    let encoded = copy.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let mut shared_mem = match winsafe::HGLOBAL::GlobalAlloc(Some(co::GMEM::FIXED), encoded.len() * size_of::<u16>()) {
        Ok(guard) => guard,
        Err(e) => return error!("error in GlobalAlloc: {e:?}"),
    };

    let mut mem_region = match shared_mem.GlobalLock() {
        Ok(guard) => guard,
        Err(e) => return error!("error in GlobalLock: {e:?}"),
    };

    mem_region.as_mut_slice().copy_from_slice(bytemuck::cast_slice(&encoded));
    drop(mem_region);

    match unsafe { winsafe::SetClipboardData(co::CF::UNICODETEXT, shared_mem.leak().ptr() as *mut _) } {
        Ok(_) => {}
        Err(e) => return error!("error in SetClipboardData: {e:?}"),
    }

    input_two_key(co::VK::LCONTROL, co::VK::CHAR_V);
}

fn input_one_key(key1: co::VK) {
    match winsafe::SendInput(&[
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key1,
                ..Default::default()
            },
        ),
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key1,
                dwFlags: co::KEYEVENTF::KEYUP,
                ..Default::default()
            },
        ),
    ]) {
        Ok(_) => {}
        Err(e) => return error!("error in SendInput: {e:?}"),
    }
}

fn input_two_key(key1: co::VK, key2: co::VK) {
    match winsafe::SendInput(&[
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key1,
                ..Default::default()
            },
        ),
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key2,
                ..Default::default()
            },
        ),
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key2,
                dwFlags: co::KEYEVENTF::KEYUP,
                ..Default::default()
            },
        ),
        HwKbMouse::Kb(
            KEYBDINPUT {
                wVk: key1,
                dwFlags: co::KEYEVENTF::KEYUP,
                ..Default::default()
            },
        ),
    ]) {
        Ok(_) => {}
        Err(e) => return error!("error in SendInput: {e:?}"),
    }
}

fn char_to_key_code(c: char) -> (co::VK, bool) {
    match c {
        '0' => (co::VK::CHAR_0, false),
        '1' => (co::VK::CHAR_1, false),
        '2' => (co::VK::CHAR_2, false),
        '3' => (co::VK::CHAR_3, false),
        '4' => (co::VK::CHAR_4, false),
        '5' => (co::VK::CHAR_5, false),
        '6' => (co::VK::CHAR_6, false),
        '7' => (co::VK::CHAR_7, false),
        '8' => (co::VK::CHAR_8, false),
        '9' => (co::VK::CHAR_9, false),

        'a' => (co::VK::CHAR_A, false),
        'b' => (co::VK::CHAR_B, false),
        'c' => (co::VK::CHAR_C, false),
        'd' => (co::VK::CHAR_D, false),
        'e' => (co::VK::CHAR_E, false),
        'f' => (co::VK::CHAR_F, false),
        'g' => (co::VK::CHAR_G, false),
        'h' => (co::VK::CHAR_H, false),
        'i' => (co::VK::CHAR_I, false),
        'j' => (co::VK::CHAR_J, false),
        'k' => (co::VK::CHAR_K, false),
        'l' => (co::VK::CHAR_L, false),
        'm' => (co::VK::CHAR_M, false),
        'n' => (co::VK::CHAR_N, false),
        'o' => (co::VK::CHAR_O, false),
        'p' => (co::VK::CHAR_P, false),
        'q' => (co::VK::CHAR_Q, false),
        'r' => (co::VK::CHAR_R, false),
        's' => (co::VK::CHAR_S, false),
        't' => (co::VK::CHAR_T, false),
        'u' => (co::VK::CHAR_U, false),
        'v' => (co::VK::CHAR_V, false),
        'w' => (co::VK::CHAR_W, false),
        'x' => (co::VK::CHAR_X, false),
        'y' => (co::VK::CHAR_Y, false),
        'z' => (co::VK::CHAR_Z, false),

        'A' => (co::VK::CHAR_A, true),
        'B' => (co::VK::CHAR_B, true),
        'C' => (co::VK::CHAR_C, true),
        'D' => (co::VK::CHAR_D, true),
        'E' => (co::VK::CHAR_E, true),
        'F' => (co::VK::CHAR_F, true),
        'G' => (co::VK::CHAR_G, true),
        'H' => (co::VK::CHAR_H, true),
        'I' => (co::VK::CHAR_I, true),
        'J' => (co::VK::CHAR_J, true),
        'K' => (co::VK::CHAR_K, true),
        'L' => (co::VK::CHAR_L, true),
        'M' => (co::VK::CHAR_M, true),
        'N' => (co::VK::CHAR_N, true),
        'O' => (co::VK::CHAR_O, true),
        'P' => (co::VK::CHAR_P, true),
        'Q' => (co::VK::CHAR_Q, true),
        'R' => (co::VK::CHAR_R, true),
        'S' => (co::VK::CHAR_S, true),
        'T' => (co::VK::CHAR_T, true),
        'U' => (co::VK::CHAR_U, true),
        'V' => (co::VK::CHAR_V, true),
        'W' => (co::VK::CHAR_W, true),
        'X' => (co::VK::CHAR_X, true),
        'Y' => (co::VK::CHAR_Y, true),
        'Z' => (co::VK::CHAR_Z, true),

        _ => (co::VK::NoValue, false),
    }
}
