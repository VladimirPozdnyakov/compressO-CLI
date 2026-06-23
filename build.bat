@echo off
REM CompressO CLI build script for Windows (CMD)
REM
REM Usage:  build.bat

setlocal enabledelayedexpansion

cd /d "%~dp0"

echo Building CompressO CLI...
cargo build --release
if errorlevel 1 (
    echo Build failed.
    exit /b 1
)

echo.
echo Build complete!
echo Binary location: target\release\compresso.exe
