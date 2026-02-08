# Windows Build Guide for StackSight

## Prerequisites

### On Windows:
1. Install Rust: https://rustup.rs/
2. Install Visual Studio Build Tools (for MSVC toolchain)
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"
3. Place your `icon.ico` in `frontend/assets/`

### Cross-compiling from Linux to Windows:
1. Install Windows target:
   ```bash
   rustup target add x86_64-pc-windows-gnu
   ```

2. Install MinGW cross-compiler:
   ```bash
   # On Ubuntu/Debian
   sudo apt install mingw-w64
   
   # On Arch
   sudo pacman -S mingw-w64-gcc
   ```

3. Configure Cargo to use the cross-compiler:
   ```bash
   # Already configured in .cargo/config.toml
   ```

## Building

### Native Windows Build (on Windows):
```powershell
# Build frontend
cd frontend
cargo build --release

# Build backend
cd ../backend
cargo build --release

# Executables will be in:
# target/release/devenv-frontend.exe
# target/release/devenv-backend.exe
```

### Cross-compile from Linux:
```bash
# Build frontend for Windows
cd frontend
cargo build --release --target x86_64-pc-windows-gnu

# Build backend for Windows
cd ../backend
cargo build --release --target x86_64-pc-windows-gnu

# Executables will be in:
# target/x86_64-pc-windows-gnu/release/devenv-frontend.exe
# target/x86_64-pc-windows-gnu/release/devenv-backend.exe
```

## Windows-Specific Features

### Icon
- The `icon.ico` file is automatically embedded into the Windows executable
- The icon appears in:
  - Taskbar
  - Alt+Tab switcher
  - Window title bar
  - File explorer (exe file icon)

### No Console Window
- In **release** builds, the frontend runs without a console window (GUI-only)
- In **debug** builds, a console window appears for logging
- The backend always shows a console (it's a service/daemon)

### Paths
- Environment paths use Windows-style format: `%USERPROFILE%\.virtualenvs\...`
- Docker paths use Windows named pipes or TCP (handled by Bollard)

## Testing on Windows

### Requirements:
1. **Docker Desktop for Windows** must be installed
2. Windows 10/11 with WSL2 (for Docker)
3. .NET Framework (for some system metrics)

### Run:
```powershell
# From the frontend directory
cargo run --release

# Or run the built executable directly
../target/release/devenv-frontend.exe
```

## Packaging for Distribution

### Single Executable
The current setup creates standalone executables that include:
- Application code
- Embedded icon
- Static resources

Users just need to:
1. Download the `.exe` file
2. Double-click to run

### Optional: Create Installer
For a more professional distribution, consider:
- **Inno Setup** (free): https://jrsoftware.org/isinfo.php
- **WiX Toolset**: https://wixtoolset.org/
- **NSIS**: https://nsis.sourceforge.io/

### Code Signing (Optional)
To avoid Windows SmartScreen warnings:
1. Get a code signing certificate
2. Sign the executable with `signtool.exe`

## Cross-Platform Compatibility

The app is designed to work on:
- ✅ Windows 10/11 (x64)
- ✅ Linux (x64)
- ✅ macOS (Intel & Apple Silicon)

All platform-specific code is properly gated with `#[cfg]` attributes.

## Troubleshooting

### "cannot find -lwindowsapp" error
Install the Windows SDK or use the MSVC toolchain instead of GNU.

### Docker connection issues on Windows
Ensure Docker Desktop is running and set to expose the daemon:
- Open Docker Desktop settings
- Enable "Expose daemon on tcp://localhost:2375 without TLS"

### Icon not showing
Make sure `icon.ico` exists at `frontend/assets/icon.ico` before building.

### Console window appears in release build
Check that you're building with `--release` flag and that the `#![windows_subsystem = "windows"]` attribute is present in `main.rs`.
