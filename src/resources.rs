use std::fs::{read_to_string, write};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use flate2::bufread::GzDecoder;
use log::info;
use tar::Archive;
use crate::global::get_appdata_dir;

static RESOURCES_TAR_GZ: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/resources.tar.gz"));
static RESOURCES_HASH: &str = env!("RESOURCES_HASH");

pub(crate) fn init() {
    info!("resources hash: {}", RESOURCES_HASH);
    let resources = get_resources_dir();
    if check_hash(&resources.join("hash")) {
        info!("hash matched. skipping expanding resources");
    } else {
        info!("hash mismatch. expanding resources...");
        expand_resources(&resources);
        info!("expanded resources");
    }
}

fn expand_resources(resources: &Path) {
    let tar = GzDecoder::new(Cursor::new(RESOURCES_TAR_GZ));
    let mut archive = Archive::new(tar);
    archive.unpack(resources).expect("unpacking resources");
    write(resources.join("hash"), RESOURCES_HASH).expect("writing hash");
}

fn check_hash(hash_file: &Path) -> bool {
    read_to_string(hash_file).unwrap_or_default().trim() == RESOURCES_HASH
}

pub fn get_resources_dir() -> PathBuf {
    return get_appdata_dir().join("resources");
}
