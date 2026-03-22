#[cfg(windows)]
fn main() {
    // Only compile resources on Windows builds
    use std::path::Path;

    let mut res = winres::WindowsResource::new();

    // Disable default VERSIONINFO generation
    res.set("ProductName", "");
    res.set("FileDescription", "");
    res.set("FileVersion", "");
    res.set("ProductVersion", "");
    // winres won't generate a VERSIONINFO block if some fields are empty, or we can just append an empty resource

    // Set application icon
    let icon_path = Path::new("assets/icon.ico");
    if icon_path.exists() {
        res.set_icon(icon_path.to_str().unwrap());
    }

    // Compile the resources
    if let Err(e) = res.compile() {
        eprintln!("Failed to compile Windows resources: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {
    // On non-Windows platforms, nothing to do
}
