use flate2::read::GzDecoder;
use std::env::args_os;
use std::ffi::OsStr;
use std::io::{Cursor, Read};
use std::process::exit;

static LICENSES_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/licenses.gz"));

pub(crate) fn licenses_reader() -> impl Read {
    GzDecoder::new(Cursor::new(LICENSES_GZ))
}

pub(crate) fn check_and_print_exit() {
    if args_os().nth(1).as_deref() == Some(OsStr::new("licenses")) {
        std::io::copy(&mut licenses_reader(), &mut std::io::stdout()).expect("writing to stdout");
        exit(0);
    }
}
