#Requires -RunAsAdministrator
# Set up a complete AegisPDF development environment on Windows.
# Run from an Administrator PowerShell:
#   Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
#   .\scripts\setup-dev-windows.ps1

param (
    [string]$PdfiumVersion = "6611"
)

$ErrorActionPreference = "Stop"

function Info  { Write-Host "[INFO] $args" -ForegroundColor Cyan }
function Ok    { Write-Host "[ OK ] $args" -ForegroundColor Green }
function Warn  { Write-Host "[WARN] $args" -ForegroundColor Yellow }
function Err   { Write-Host "[ERR ] $args" -ForegroundColor Red; exit 1 }

# ─────────────────────────────────────────────────────────────────────────────
# 1. winget / scoop helpers
# ─────────────────────────────────────────────────────────────────────────────
function Install-Winget {
    param([string]$Id)
    try {
        winget install --id $Id --exact --accept-source-agreements --accept-package-agreements -h
        Ok "$Id installed"
    } catch {
        Warn "winget install $Id failed: $_"
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 2. Visual Studio Build Tools (C++ workload)  — needed by Tauri / pdfium-sys
# ─────────────────────────────────────────────────────────────────────────────
Info "Checking Visual Studio Build Tools..."
$vsPath = & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" `
    -products * -latest -requires Microsoft.VisualStudio.Workload.NativeDesktop `
    -property installationPath 2>$null
if (-not $vsPath) {
    Info "Installing Visual Studio Build Tools 2022 (C++ workload)..."
    Install-Winget "Microsoft.VisualStudio.2022.BuildTools"
    Warn "If the installer launched, complete it, then re-run this script."
} else {
    Ok "Visual Studio Build Tools found at $vsPath"
}

# ─────────────────────────────────────────────────────────────────────────────
# 3. WebView2 Runtime
# ─────────────────────────────────────────────────────────────────────────────
$wv2 = Get-ItemProperty -Path "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" `
       -ErrorAction SilentlyContinue
if (-not $wv2) {
    Info "Installing Microsoft Edge WebView2 Runtime..."
    Install-Winget "Microsoft.EdgeWebView2Runtime"
} else {
    Ok "WebView2 Runtime already installed"
}

# ─────────────────────────────────────────────────────────────────────────────
# 4. Node.js 20
# ─────────────────────────────────────────────────────────────────────────────
if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Info "Installing Node.js 20..."
    Install-Winget "OpenJS.NodeJS.LTS"
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" +
                [System.Environment]::GetEnvironmentVariable("PATH", "User")
} else {
    Ok "Node.js $(node --version) already present"
}

# ─────────────────────────────────────────────────────────────────────────────
# 5. Rust
# ─────────────────────────────────────────────────────────────────────────────
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    Info "Installing Rust via rustup..."
    $rustupUrl = "https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"
    $rustupExe = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest $rustupUrl -OutFile $rustupExe
    & $rustupExe -y --no-modify-path
    Remove-Item $rustupExe -Force
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
} else {
    Ok "Rust $(rustc --version) already present"
}

rustup update stable

# ─────────────────────────────────────────────────────────────────────────────
# 6. WiX Toolset v3 (for .msi)
# ─────────────────────────────────────────────────────────────────────────────
if (-not (Get-Command candle -ErrorAction SilentlyContinue)) {
    Info "Installing WiX Toolset v3..."
    Install-Winget "WiXToolset.WiXToolset"
} else {
    Ok "WiX Toolset already installed"
}

# ─────────────────────────────────────────────────────────────────────────────
# 7. NSIS (for -setup.exe)
# ─────────────────────────────────────────────────────────────────────────────
if (-not (Get-Command makensis -ErrorAction SilentlyContinue)) {
    Info "Installing NSIS..."
    Install-Winget "NSIS.NSIS"
} else {
    Ok "NSIS already installed"
}

# ─────────────────────────────────────────────────────────────────────────────
# 8. PDFium shared library
# ─────────────────────────────────────────────────────────────────────────────
$pdfiumDest = "$env:SystemRoot\System32\pdfium.dll"
if (-not (Test-Path $pdfiumDest)) {
    Info "Downloading PDFium v$PdfiumVersion..."
    $pdfiumUrl = "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F$PdfiumVersion/pdfium-win-x64.tgz"
    $tmp = "$env:TEMP\pdfium"
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null
    Invoke-WebRequest $pdfiumUrl -OutFile "$tmp\pdfium.tgz"
    tar -xzf "$tmp\pdfium.tgz" -C $tmp
    Copy-Item "$tmp\bin\pdfium.dll" $pdfiumDest -Force
    Remove-Item $tmp -Recurse -Force
    Ok "PDFium installed to $pdfiumDest"
} else {
    Ok "PDFium already present at $pdfiumDest"
}

# ─────────────────────────────────────────────────────────────────────────────
# 9. Tesseract OCR
# ─────────────────────────────────────────────────────────────────────────────
if (-not (Get-Command tesseract -ErrorAction SilentlyContinue)) {
    Info "Installing Tesseract OCR..."
    Install-Winget "UB-Mannheim.TesseractOCR"
    $tessPath = "C:\Program Files\Tesseract-OCR"
    if (Test-Path $tessPath) {
        [System.Environment]::SetEnvironmentVariable("PATH",
            "$tessPath;$([System.Environment]::GetEnvironmentVariable('PATH','Machine'))",
            "Machine")
        $env:PATH = "$tessPath;$env:PATH"
    }
} else {
    Ok "Tesseract already installed: $(tesseract --version 2>&1 | Select-Object -First 1)"
}

# ─────────────────────────────────────────────────────────────────────────────
# 10. npm dependencies
# ─────────────────────────────────────────────────────────────────────────────
Info "Installing npm dependencies..."
Push-Location (Split-Path $PSScriptRoot -Parent)
npm install
Pop-Location

# ─────────────────────────────────────────────────────────────────────────────
# 11. Done
# ─────────────────────────────────────────────────────────────────────────────
Write-Host ""
Ok "Development environment ready!"
Write-Host ""
Write-Host "  Run dev mode:     npm run tauri dev" -ForegroundColor Cyan
Write-Host "  Build MSI:        npm run tauri build -- --bundles msi" -ForegroundColor Cyan
Write-Host "  Build NSIS:       npm run tauri build -- --bundles nsis" -ForegroundColor Cyan
Write-Host "  Build all:        npm run tauri build" -ForegroundColor Cyan
Write-Host ""
