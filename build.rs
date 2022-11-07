use std::env::var_os;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use flate2::Compression;
use flate2::write::GzEncoder;
use sha2::{Sha256, Digest};

fn main() {
    // default config settings
    if cfg!(debug_assertions) && cfg!(feature = "default") {
        println!(r#"cargo:rustc-cfg=feature="debug_window""#);
    }
    
    pack_resources();
    hash_resources();
}

fn out_dir(joins: impl AsRef<Path>) -> PathBuf {
    let mut path_buf = PathBuf::from(var_os("OUT_DIR").expect("OUT_DIR"));
    path_buf.push(joins);
    path_buf
}

fn manifest_dir(joins: impl AsRef<Path>) -> PathBuf {
    let mut path_buf = PathBuf::from(var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    path_buf.push(joins);
    path_buf
}

fn pack_resources() {
    let tar_gz = File::create(out_dir("resources.tar.gz")).expect("create archive");
    let enc = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all("", manifest_dir("resources"))
        .expect("appending to tar");
    tar.finish().expect("appending to tar");
}

fn hash_resources() {
    let mut tar_gz = File::open(out_dir("resources.tar.gz")).expect("open archive");
    let hash = hash_read(&mut tar_gz).expect("hashing");
    println!("cargo:rustc-env=RESOURCES_HASH={}", hex::encode(hash));
}


fn hash_read(mut read: impl io::Read) -> io::Result<Vec<u8>> {
    let mut hasher = Sha256::new();
    let mut buf = [0 as u8; 8 * 1024];
    loop {
        match read.read(&mut buf) {
            Ok(0) => break,
            Ok(size) => hasher.update(&buf[..size]),
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => (),
            Err(e) => return Err(e),
        }
    }

    Ok(Vec::from(&hasher.finalize()[..]))
}
