@echo off
echo ====================================
echo StackSight WebView2 Installer
echo ====================================
echo.

REM Check if WebView2 is already installed
reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" >nul 2>&1
if %errorlevel% equ 0 (
    echo WebView2 is already installed!
    echo Starting StackSight...
    echo.
    goto :start_app
)

echo WebView2 Runtime is NOT installed.
echo.
echo Downloading WebView2 Runtime installer...
echo.

REM Download WebView2 Runtime installer
powershell -Command "& {Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile 'MicrosoftEdgeWebview2Setup.exe'}"

if not exist "MicrosoftEdgeWebview2Setup.exe" (
    echo.
    echo ERROR: Failed to download WebView2 installer.
    echo Please install manually from:
    echo https://developer.microsoft.com/en-us/microsoft-edge/webview2/
    echo.
    pause
    exit /b 1
)

echo.
echo Installing WebView2 Runtime...
echo (This may take a minute)
echo.

REM Install WebView2
start /wait MicrosoftEdgeWebview2Setup.exe /silent /install

REM Clean up installer
del MicrosoftEdgeWebview2Setup.exe

echo.
echo WebView2 Runtime installed successfully!
echo.

:start_app
echo Starting StackSight...
echo.

REM Start the backend server
echo Starting backend server...
start "" "devenv-backend.exe"

REM Wait for backend to start
timeout /t 2 /nobreak >nul

REM Start the frontend GUI
echo Starting frontend...
start "" "devenv-frontend.exe"

echo.
echo StackSight is running!
echo.
echo Backend: http://127.0.0.1:8765
echo Frontend: GUI window should appear
echo.
echo To stop: Close the GUI window and backend console
echo.
pause
