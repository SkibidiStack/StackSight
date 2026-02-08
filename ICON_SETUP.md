# Setting Up Your StackSight Icon

## Current Status

✅ Your app is **fully configured** to use custom icons on all platforms!
✅ **Windows compilation verified** - both frontend and backend compile successfully

## What You Need To Do

### 1. Place Your Icon Files

You already have your StackSight logo. Now you need to place the icon files:

```
frontend/assets/
├── icon.png       # For Linux/macOS (256x256 or larger)
└── icon.ico       # For Windows (contains multiple sizes)
```

### 2. Icon File Requirements

#### For `icon.png` (Linux/macOS):
- Format: PNG with transparency
- Recommended size: 256x256 pixels or larger
- Used at runtime on Linux
- Embedded in macOS app bundles

#### For `icon.ico` (Windows):
- Format: Windows ICO file with multiple embedded sizes
- Required sizes: 16x16, 32x32, 48x48, 64x64, 128x128, 256x256
- Embedded into the `.exe` at compile time

### 3. Converting Your Logo to ICO

If you already have `icon.ico` files created, just copy them to `frontend/assets/icon.ico`.

If you need to create them from your PNG logo:

**Using ImageMagick (Linux/macOS):**
```bash
cd /home/shaun/StackSight/StackSight/frontend/assets
convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

**Using Online Tools:**
- https://icoconvert.com/
- https://convertio.co/png-ico/
- Upload your PNG, select all sizes, download ICO

**Using GIMP:**
1. Open your PNG in GIMP
2. File → Export As
3. Change extension to `.ico`
4. Select all size options
5. Export

### 4. Verify Files Are In Place

```bash
ls -lh /home/shaun/StackSight/StackSight/frontend/assets/
```

You should see:
```
icon.png  (your logo for Linux/macOS)
icon.ico  (your logo for Windows with multiple sizes)
```

### 5. Build and Test

#### Test on Linux (current platform):
```bash
cd /home/shaun/StackSight/StackSight/frontend
cargo run --release
```

The icon should appear in your taskbar!

#### Build for Windows:
```bash
cd /home/shaun/StackSight/StackSight/frontend
cargo build --release --target x86_64-pc-windows-gnu
```

The `.exe` file will be at:
```
../target/x86_64-pc-windows-gnu/release/devenv-frontend.exe
```

Copy this to a Windows machine and run it - your icon will be embedded!

## How It Works

### On Linux/macOS:
- `main.rs` loads `icon.png` at runtime
- Uses the `image` crate to decode PNG
- Passes RGBA data to the window manager
- Icon appears in taskbar/dock

### On Windows:
- `build.rs` runs during compilation
- Uses `winres` crate to embed `icon.ico` into the executable
- Icon becomes part of the `.exe` file metadata
- Windows automatically shows it in:
  - Taskbar
  - Alt+Tab switcher
  - Window title bar
  - File explorer (exe icon)
  - Task Manager

## Troubleshooting

### Icon doesn't appear on Linux:
- Check that `icon.png` exists at `frontend/assets/icon.png`
- Check terminal output for warning: "Icon file not found"
- Try a larger image (512x512)

### Icon doesn't appear on Windows:
- Ensure `icon.ico` existed **before building**
- The icon is embedded at compile-time, not runtime
- Rebuild if you add the icon after building
- Check build output for "Failed to compile Windows resources"

### Build fails with "icon file not found":
- On Windows builds, `build.rs` looks for `icon.ico`
- If missing, build will fail
- Either add the icon or remove the icon line from `build.rs`

## Next Steps

1. ✅ **Copy your icon files** to `frontend/assets/`
2. ✅ **Rebuild** the app: `cargo build --release`
3. ✅ **Test** that the icon appears in the taskbar
4. ✅ **Build for Windows** if you want to test on Windows too

Your app will look professional with the custom StackSight branding!
