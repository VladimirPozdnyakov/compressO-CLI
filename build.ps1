# CompressO CLI build script for Windows (PowerShell)
#
# Usage:  .\build.ps1

$ErrorActionPreference = "Stop"

Set-Location -Path $PSScriptRoot

Write-Host "Building CompressO CLI..."
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Error "Build failed."
    exit $LASTEXITCODE
}

Write-Host ""
Write-Host "Build complete!"
Write-Host "Binary location: target\release\compresso.exe"
