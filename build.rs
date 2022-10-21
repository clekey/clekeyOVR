use std::env::var;

fn main() {
    if cfg!(debug_assertions) && cfg!(feature = "default") {
        println!(r#"cargo:rustc-cfg=feature="debug_window""#);
    }
}
