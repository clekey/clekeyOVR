use std::env::var;

fn main() {
    if cfg!(debug_assertions) && cfg!(feature = "default") {
        println!(r#"cargo:rustc-cfg=feature="debug_window""#);
    }
    if var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") && cfg!(feature = "default") {
        println!(r#"cargo:rustc-cfg=feature="openvr""#);
    }
}
