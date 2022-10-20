use cfg_if::cfg_if;
macro_rules! import {
    ($name: ident) => {
        mod $name;
        pub use $name::*;
    };
}
cfg_if! {
    if #[cfg(windows)] {
        import!(win);
    } else {
        import!(mock);
    }
}
