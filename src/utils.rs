use glam::{UVec2, Vec2};
use std::ffi::{CString, OsString};
use std::path::PathBuf;

pub trait IntoStringLossy {
    /// always convert to string with into_string and to_string_lossy
    fn into_string_lossy(self) -> String;
}

impl IntoStringLossy for PathBuf {
    fn into_string_lossy(self) -> String {
        self.into_os_string().into_string_lossy()
    }
}

impl IntoStringLossy for OsString {
    fn into_string_lossy(self) -> String {
        self.into_string()
            .unwrap_or_else(|x| x.to_string_lossy().into_owned())
    }
}

pub trait ToTuple {
    type Tuple;
    fn to_tuple(&self) -> Self::Tuple;
}

impl ToTuple for Vec2 {
    type Tuple = (f32, f32);

    fn to_tuple(&self) -> Self::Tuple {
        (self.x, self.y)
    }
}

impl ToTuple for UVec2 {
    type Tuple = (u32, u32);

    fn to_tuple(&self) -> Self::Tuple {
        (self.x, self.y)
    }
}

pub trait ToCString {
    /// always convert to string with into_string and to_string_lossy
    fn to_c_string(&self) -> CString;
}

impl ToCString for String {
    fn to_c_string(&self) -> CString {
        CString::new(self.as_bytes()).unwrap()
    }
}

#[macro_export]
macro_rules! char_to_str {
    ($char: expr) => {
        unsafe {
            static ARY: [u8; 4] = $crate::utils::encode_utf8_raw($char);
            $crate::utils::encoded_to_str(&ARY)
        }
    };
}

// const stability
//noinspection RsAssertEqual
/// SAFETY: caller must guarantee bytes have valid UTF8 & suffixed with '\0' if it's not 4 bytes
pub const unsafe fn encoded_to_str(bytes: &[u8]) -> &str {
    debug_assert!(bytes.len() == 4, "bytes len is not 4");
    if bytes[1] == 0 {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(bytes.as_ptr(), 1))
    } else if bytes[2] == 0 {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(bytes.as_ptr(), 2))
    } else if bytes[3] == 0 {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(bytes.as_ptr(), 3))
    } else {
        std::str::from_utf8_unchecked(bytes)
    }
}

pub const fn encode_utf8_raw(code: char) -> [u8; 4] {
    let len = code.len_utf8();
    let code = code as u32;
    match len {
        1 => {
            let a = code as u8;
            [a, 0, 0, 0]
        }
        2 => {
            let a = (code >> 6 & 0x1F) as u8 | 0b1100_0000;
            let b = (code & 0x3F) as u8 | 0b1000_0000;
            [a, b, 0, 0]
        }
        3 => {
            let a = (code >> 12 & 0x0F) as u8 | 0b1110_0000;
            let b = (code >> 6 & 0x3F) as u8 | 0b1000_0000;
            let c = (code & 0x3F) as u8 | 0b1000_0000;
            [a, b, c, 0]
        }
        4 => {
            let a = (code >> 18 & 0x07) as u8 | 0b1111_0000;
            let b = (code >> 12 & 0x3F) as u8 | 0b1000_0000;
            let c = (code >> 6 & 0x3F) as u8 | 0b1000_0000;
            let d = (code & 0x3F) as u8 | 0b1000_0000;
            [a, b, c, d]
        }
        _ => unreachable!(),
    }
}
