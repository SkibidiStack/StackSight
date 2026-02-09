# StackSight for Windows

## IMPORTANT: Known Issue - Cross-Compilation Limitation

**The Windows build was cross-compiled from Linux and may have WebView2 linking issues.**

### Solution: Build on Windows Instead

To get a fully working version:

1. **On your Windows machine**, install:
   ```powershell
   # Install Rust
   winget install Rustlang.Rustup
   
   # Install Visual Studio Build Tools
   winget install Microsoft.VisualStudio.2022.BuildTools
   # (Select "Desktop development with C++")
   ```

2. **Copy the entire StackSight source folder** to Windows

3. **Build on Windows**:
   ```powershell
   cd StackSight\StackSight\frontend
   cargo build --release
   
   cd ..\backend
   cargo build --release
   ```

4. **Run**:
   ```powershell
   # Start backend
   ..\target\release\devenv-backend.exe
   
   # Start frontend (in new window)
   ..\target\release\devenv-frontend.exe
   ```

### Why This Happens
Cross-compiling GUI apps from Linux to Windows can miss runtime dependencies like WebView2Loader.dll. Building natively on Windows ensures all dependencies are properly linked.

## Quick Start

1. **Run StackSight**:
   - Double-click `start-stacksight.bat`
   - Or run both manually:
     1. Run `devenv-backend.exe` (console window)
     2. Run `devenv-frontend.exe` (GUI)

2. **First Launch**:
   - Setup wizard will appear
   - Follow the steps to configure paths
   - Docker detection (optional - requires Docker Desktop)
   - Language tools detection

## Files

- `devenv-backend.exe` - Backend server (runs in console)
- `devenv-frontend.exe` - GUI application
- `start-stacksight.bat` - Starts both automatically

## System Requirements

- **Windows 10/11** (64-bit)
- **RAM**: 2GB minimum
- **Storage**: 50MB for app, more for virtual environments
- **Optional**: Docker Desktop (for container management)

## Features

### Core Features (Work Immediately)
- ✅ System monitoring (CPU, RAM, disk, network)
- ✅ Process management
- ✅ Virtual environment creation
- ✅ Beautiful GUI with taskbar icon

### Optional Features (Require Installation)
- 🐳 Docker management (requires Docker Desktop)
- 🐍 Python environments (requires Python)
- 🟢 Node.js environments (requires Node.js)
- 🦀 Rust environments (requires Rust)

## Installing Optional Tools

### Docker Desktop
https://www.docker.com/products/docker-desktop/

### Python
```powershell
winget install Python.Python.3.12
```

### Node.js
```powershell
winget install OpenJS.NodeJS
```

## Configuration

Configuration files are stored at:
```
%APPDATA%\stacksight\
├── setup.json          # Setup wizard config
└── environments.json   # Virtual environments list
```

Virtual environments default location:
```
%USERPROFILE%\.virtualenvs\
```

## Troubleshooting

### "Windows protected your PC" warning
- Click "More info"
- Click "Run anyway"
- This is normal for unsigned applications

### Backend won't start
- Check if port 8765 is already in use
- Try running as administrator
- Check Windows Firewall settings

### Frontend can't connect to backend
- Ensure backend is running first
- Check that `devenv-backend.exe` console window is open
- Try restarting both

### Docker features don't work
- Install Docker Desktop
- Ensure Docker Desktop is running
- Check Docker Desktop settings

## How It Works

**Backend** (`devenv-backend.exe`):
- Runs as a WebSocket server on `ws://127.0.0.1:8765`
- Manages Docker, virtual environments, system monitoring
- Console window shows logs

**Frontend** (`devenv-frontend.exe`):
- GUI application with taskbar icon
- Connects to backend via WebSocket
- Displays data and sends commands

Both must be running for full functionality.

## Updates

To update:
1. Download new versions of the .exe files
2. Replace the old files
3. Restart the application

## Support

For issues or questions:
- Check logs in backend console window
- Configuration: `%APPDATA%\stacksight\`
- Reset config: Delete `%APPDATA%\stacksight\` folder

---

**StackSight** - Docker & DevEnv Manager
Version 0.1.0
