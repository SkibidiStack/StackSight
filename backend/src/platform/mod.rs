#[derive(Clone, Debug)]
pub enum PlatformKind {
    Windows,
    Linux,
    MacOS,
    Unknown,
}

pub fn detect() -> PlatformKind {
    if cfg!(target_os = "windows") {
        PlatformKind::Windows
    } else if cfg!(target_os = "linux") {
        PlatformKind::Linux
    } else if cfg!(target_os = "macos") {
        PlatformKind::MacOS
    } else {
        PlatformKind::Unknown
    }
}
