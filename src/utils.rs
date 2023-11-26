use glam::{UVec2, Vec2};
use std::ffi::{CString, OsString};
use std::path::{Path, PathBuf};
use std::time::Instant;

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

impl ToCString for Path {
    fn to_c_string(&self) -> CString {
        CString::new(self.to_string_lossy().into_owned().into_bytes()).unwrap()
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

pub struct FPSComputer<const AVG_FRAMES: usize> {
    time_buf: [Instant; AVG_FRAMES],
    // highest bit indicates if filling (1) or filled (0)
    cursor: u32,
    full: bool,
}

impl<const AVG_FRAMES: usize> FPSComputer<AVG_FRAMES> {
    pub fn new() -> Self {
        assert_ne!(AVG_FRAMES, 0, "AVG_FRAMES must not zero");
        assert!(
            AVG_FRAMES < u32::MAX as usize,
            "AVG_FRAMES must not be greater than u32::MAX"
        );
        Self {
            time_buf: [Instant::now(); AVG_FRAMES],
            cursor: 0,
            full: false,
        }
    }

    // returns (average, one_frame)
    pub fn on_frame(&mut self) -> (f64, f64) {
        let now = Instant::now();
        if !self.full && self.cursor == 0 {
            // nothing initialized: just init and
            self.time_buf[self.cursor as usize] = now;
            self.cursor = 1;
            if self.cursor == AVG_FRAMES as u32 {
                // if AVG_FRAMES == 1, it's filled
                self.cursor = 0;
                self.full = true;
            }
            (0.0, 0.0)
        } else {
            let avg_time;
            let last_time;
            let frames;
            if self.full {
                avg_time = self.time_buf[self.cursor as usize];
                let last_index =
                    self.cursor.checked_sub(1).unwrap_or(AVG_FRAMES as u32 - 1) as usize;
                last_time = self.time_buf[last_index];
                frames = AVG_FRAMES as f64;
            } else {
                assert_ne!(self.cursor, 0);
                avg_time = self.time_buf[0];
                last_time = self.time_buf[self.cursor as usize - 1];
                frames = self.cursor as f64;
            };
            let avg_fps = 1.0 / (now - avg_time).as_secs_f64() * frames;
            let one_fps = 1.0 / (now - last_time).as_secs_f64();
            self.time_buf[self.cursor as usize] = now;
            self.cursor += 1;
            if self.cursor == AVG_FRAMES as u32 {
                self.cursor = 0;
                self.full = true;
            }
            (avg_fps, one_fps)
        }
    }
}
