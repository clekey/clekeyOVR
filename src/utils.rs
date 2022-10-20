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
