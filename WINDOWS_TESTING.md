# StackSight - Windows Setup & Testing Guide

## Prerequisites on Windows VM

### 1. Install Rust
```powershell
# Download and run rustup-init.exe from:
# https://rustup.rs/

# Or use winget (Windows 11/10):
winget install Rustlang.Rustup

# Verify installation:
rustc --version
cargo --version
```

### 2. Install Visual Studio Build Tools
Required for compiling native dependencies:

**Option A: Visual Studio Installer**
1. Download: https://visualstudio.microsoft.com/downloads/
2. Install "Desktop development with C++" workload
3. Or just the "MSVC v143 - VS 2022 C++ x64/x86 build tools"

**Option B: Command line**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
```

### 3. Install Git (Optional but recommended)
```powershell
winget install Git.Git
```

## Building the App

### Transfer Project to Windows VM

**Option 1: Git Clone (if you pushed to repo)**
```powershell
git clone <your-repo-url>
cd StackSight
```

**Option 2: Copy via Shared Folder**
- Set up a shared folder between host and VM
- Copy the entire StackSight directory

**Option 3: Network Transfer**
```powershell
# On Linux host:
cd /home/shaun/StackSight
tar czf stacksight.tar.gz StackSight/
python3 -m http.server 8000

# On Windows VM (in PowerShell):
Invoke-WebRequest -Uri "http://<linux-ip>:8000/stacksight.tar.gz" -OutFile "stacksight.tar.gz"
tar -xzf stacksight.tar.gz
```

### Build the Application

```powershell
# Navigate to project
cd StackSight\StackSight

# Build backend (optional - if you want to test backend on Windows)
cd backend
cargo build --release
cd ..

# Build frontend (the main GUI app)
cd frontend
cargo build --release

# The executable will be at:
# target\release\devenv-frontend.exe
```

## Setting Up the Icon

Before building, make sure you have the icon file:

```powershell
# Icon should be at:
# frontend\assets\icon.ico

# Verify it exists:
Test-Path frontend\assets\icon.ico
```

If missing, copy it from your Linux machine or the asset files.

## Running the App

### Development Mode
```powershell
cd frontend
cargo run
```

### Release Build
```powershell
# After building:
.\target\release\devenv-frontend.exe

# Or create a shortcut on desktop
```

## Testing Checklist

### ✅ Visual Elements
- [ ] App icon appears in taskbar
- [ ] No console window appears (release build)
- [ ] Window title shows "StackSight - DevEnv Manager"
- [ ] UI renders correctly

### ✅ Setup Wizard
- [ ] Wizard appears on first launch
- [ ] Docker detection works (if Docker Desktop installed)
- [ ] Default paths use Windows format: `%USERPROFILE%\.virtualenvs`
- [ ] Language detection finds installed tools
- [ ] Setup config saves to: `%APPDATA%\stacksight\setup.json`

### ✅ Docker Integration (if Docker Desktop installed)
- [ ] Docker Desktop must be running
- [ ] App connects via named pipe: `//./pipe/docker_engine`
- [ ] Can list containers
- [ ] Can start/stop containers

### ✅ Virtual Environments
- [ ] Can create Python venv in `%USERPROFILE%\.virtualenvs`
- [ ] Paths display correctly with Windows separators
- [ ] Environment activation works

### ✅ System Monitoring
- [ ] CPU/Memory stats display
- [ ] Disk info shows Windows drives (C:, D:, etc.)
- [ ] Process list works

## Installing Docker Desktop (Optional)

For full Docker functionality testing:

1. **Download Docker Desktop for Windows**
   - https://www.docker.com/products/docker-desktop/

2. **Install and Configure**
   ```powershell
   # After installation, start Docker Desktop
   # Enable WSL2 backend if available
   ```

3. **Verify**
   ```powershell
   docker ps
   docker version
   ```

## Installing Development Tools (for testing detection)

Test the wizard's language detection by installing:

### Python
```powershell
winget install Python.Python.3.12
python --version
```

### Node.js
```powershell
winget install OpenJS.NodeJS
node --version
```

### Rust (already installed)
```powershell
rustc --version
```

### Go (optional)
```powershell
winget install GoLang.Go
go version
```

### Java (optional)
```powershell
winget install Microsoft.OpenJDK.17
java --version
```

## Troubleshooting

### Build Errors

**"link.exe not found"**
- Install Visual Studio Build Tools (see Prerequisites)
- Restart terminal after installation

**"failed to run custom build command for `winres`"**
- Ensure `icon.ico` exists at `frontend\assets\icon.ico`
- If missing, comment out icon lines in `frontend\build.rs`

**"cannot find -lwindowsapp"**
- Use MSVC toolchain (not GNU):
  ```powershell
  rustup default stable-msvc
  ```

### Runtime Issues

**Console window appears in release build**
- Make sure you built with `--release` flag
- Check that `#![windows_subsystem = "windows"]` is in `main.rs`

**Icon doesn't appear**
- Icon is embedded at compile time
- Rebuild after adding `icon.ico`
- Check Windows Explorer → Right-click EXE → Properties → should show icon

**Docker connection fails**
- Ensure Docker Desktop is running
- Check Docker settings: Expose daemon on tcp://localhost:2375 (if using HTTP)
- Or ensure named pipe is enabled (default)

**App crashes on startup**
- Run in debug mode to see error:
  ```powershell
  cargo run
  ```
- Check logs in console output

### Configuration Files

Configuration is stored at:
```
%APPDATA%\stacksight\
├── setup.json          # Setup wizard config
└── environments.json   # Virtual environments
```

To reset:
```powershell
Remove-Item -Recurse -Force $env:APPDATA\stacksight
```

## Cross-Compilation from Linux (Alternative)

If you prefer to build on Linux for Windows:

```bash
# On your Linux machine:
cd /home/shaun/StackSight/StackSight/frontend

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# Copy to Windows VM:
# target/x86_64-pc-windows-gnu/release/devenv-frontend.exe
```

Then just copy the `.exe` to your Windows VM.

## Performance Notes

- **First build** will take 10-20 minutes (downloads and compiles dependencies)
- **Subsequent builds** are much faster (1-3 minutes)
- **Release builds** are optimized and smaller than debug builds
- The exe is standalone - no DLLs needed (except system libraries)

## Creating a Distributable

For sharing or testing on other Windows machines:

```powershell
# After building in release mode:
cd target\release

# The exe is standalone, just copy it:
Copy-Item devenv-frontend.exe $env:USERPROFILE\Desktop\

# Or create an installer (optional):
# Use Inno Setup, NSIS, or WiX
```

## Next Steps

1. **Build the app** following the steps above
2. **Run it** and verify the setup wizard appears
3. **Test all features** using the checklist
4. **Report any Windows-specific issues** you find

Need help? Check:
- Build errors → See Troubleshooting section
- Missing dependencies → Verify Prerequisites
- Runtime issues → Run in debug mode for logs
