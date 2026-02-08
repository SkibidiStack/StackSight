#[cfg(windows)]
fn main() {
    // Only compile resources on Windows builds
    use std::path::Path;
    
    let mut res = winres::WindowsResource::new();
    
    // Backend runs as a service/daemon, so we want console output
    // No icon needed for backend service
    
    // Set application metadata
    res.set("ProductName", "StackSight DevEnv Backend");
    res.set("FileDescription", "StackSight DevEnv Manager Backend Service");
    res.set("CompanyName", "StackSight Team");
    res.set("LegalCopyright", "Copyright © 2026 StackSight Team");
    
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
