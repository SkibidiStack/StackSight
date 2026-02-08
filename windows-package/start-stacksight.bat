@echo off
echo Starting StackSight DevEnv Manager...
echo.

REM Start the backend server
echo Starting backend server...
start "" "devenv-backend.exe"

REM Wait a moment for backend to start
timeout /t 2 /nobreak >nul

REM Start the frontend GUI
echo Starting frontend...
start "" "devenv-frontend.exe"

echo.
echo StackSight is starting...
echo Backend: http://127.0.0.1:8765
echo Frontend: GUI window should appear
echo.
echo To stop: Close the GUI window and backend console
