# App Icon Setup

## Steps to set up your StackSight logo as the app icon:

1. **Save your logo here** as `icon.png`
   - Recommended size: 256x256 pixels or larger
   - Format: PNG with transparency

2. **Optional: Generate multi-resolution icons** (for better quality on different displays):
   ```bash
   # Convert to ICO format with multiple sizes (Windows/Linux)
   convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
   
   # Generate macOS icon set
   mkdir icon.iconset &&
   sips -z 16 16 icon.png --out icon.iconset/icon_16x16.png \
   sips -z 32 32 icon.png --out icon.iconset/icon_16x16@2x.png \
   sips -z 32 32 icon.png --out icon.iconset/icon_32x32.png\
   sips -z 64 64 icon.png --out icon.iconset/icon_32x32@2x.png\
   sips -z 128 128 icon.png --out icon.iconset/icon_128x128.png\
   sips -z 256 256 icon.png --out icon.iconset/icon_128x128@2x.png\
   sips -z 256 256 icon.png --out icon.iconset/icon_256x256.png\
   sips -z 512 512 icon.png --out icon.iconset/icon_256x256@2x.png\
   sips -z 512 512 icon.png --out icon.iconset/icon_512x512.png\
   cp icon.png icon.iconset/icon_512x512@2x.png\
   iconutil -c icns icon.iconset
   ```

3. **Build and test**:
   ```bash
   cd /home/shaun/StackSight/StackSight/frontend
   cargo build --release
   ```

The app will now use your StackSight logo in the taskbar!

## Current Implementation

The `main.rs` file now includes:
- Icon loading from `assets/icon.png`
- Automatic conversion to the format required by the window manager
- Graceful fallback if the icon is missing (with a warning log)
- Updated window title to "StackSight - DevEnv Manager"

## File Locations

- **Development**: `frontend/assets/icon.png`
- **For production builds**, you may want to embed the icon using `include_bytes!` macro for a truly standalone executable.
