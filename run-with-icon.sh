#!/bin/bash
# Helper script to run the app with proper icon in KDE/Linux

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Set environment variables for Linux window managers
export GDK_BACKEND=x11
export WM_CLASS="stacksight-devenv-manager"

# Run the frontend
cd "$SCRIPT_DIR/frontend"
cargo run --release
