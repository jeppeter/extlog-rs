

#[cfg(target_os = "windows")]
pub (crate) mod wchar_windows;
#[cfg(target_os = "windows")]
pub (crate) mod loglib_windows;

pub mod loglib;