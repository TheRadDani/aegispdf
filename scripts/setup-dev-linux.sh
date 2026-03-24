#!/usr/bin/env bash
# Set up a complete AegisPDF development environment on Debian/Ubuntu.
# Tested on Ubuntu 22.04 LTS and Ubuntu 24.04 LTS.
#
# Run as normal user (will call sudo internally):
#   chmod +x scripts/setup-dev-linux.sh && ./scripts/setup-dev-linux.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$SCRIPT_DIR/.."

info()  { echo -e "\033[1;34m[INFO]\033[0m  $*"; }
ok()    { echo -e "\033[1;32m[ OK ]\033[0m  $*"; }
warn()  { echo -e "\033[1;33m[WARN]\033[0m  $*"; }
die()   { echo -e "\033[1;31m[ERR ]\033[0m  $*" >&2; exit 1; }

# ────────────────────────────────────────────────────────────────────
# 1. System packages
# ────────────────────────────────────────────────────────────────────
info "Updating apt..."
sudo apt-get update -q

info "Installing Tauri system prerequisites..."
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  pkg-config \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev

info "Installing icon generation tools..."
sudo apt-get install -y \
  inkscape \
  imagemagick \
  icoutils \
  icnsutils \
  librsvg2-bin

info "Installing OCR runtime..."
sudo apt-get install -y tesseract-ocr tesseract-ocr-eng

info "Installing optional packaging tools..."
sudo apt-get install -y \
  rpm \
  squashfs-tools \
  fakeroot

# ────────────────────────────────────────────────────────────────────
# 2. Node.js (via nvm or system)
# ────────────────────────────────────────────────────────────────────
if ! command -v node &>/dev/null; then
  info "Installing Node.js 20 via NodeSource..."
  curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
  sudo apt-get install -y nodejs
fi
ok "Node.js $(node --version)"

# ────────────────────────────────────────────────────────────────────
# 3. Rust toolchain
# ────────────────────────────────────────────────────────────────────
if ! command -v rustup &>/dev/null; then
  info "Installing Rust via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
  # shellcheck source=/dev/null
  source "$HOME/.cargo/env"
fi
ok "Rust $(rustc --version)"
rustup update stable
rustup component add clippy rustfmt

# ────────────────────────────────────────────────────────────────────
# 4. Tauri CLI
# ────────────────────────────────────────────────────────────────────
if ! command -v cargo-tauri &>/dev/null; then
  info "Installing @tauri-apps/cli via cargo..."
  cargo install tauri-cli --version '^2' --locked
fi
ok "tauri-cli installed"

# ────────────────────────────────────────────────────────────────────
# 5. PDFium shared library
# ────────────────────────────────────────────────────────────────────
PDFIUM_VERSION="6611"
PDFIUM_URL="https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F${PDFIUM_VERSION}/pdfium-linux-x64.tgz"
PDFIUM_LIB="/usr/local/lib/libpdfium.so"

if [[ ! -f "$PDFIUM_LIB" ]]; then
  info "Downloading PDFium v${PDFIUM_VERSION}..."
  TMP="$(mktemp -d)"
  wget -q "$PDFIUM_URL" -O "$TMP/pdfium.tgz"
  tar -xzf "$TMP/pdfium.tgz" -C "$TMP"
  sudo cp "$TMP/lib/libpdfium.so" /usr/local/lib/
  sudo ldconfig
  rm -rf "$TMP"
  ok "PDFium installed to $PDFIUM_LIB"
else
  ok "PDFium already present at $PDFIUM_LIB"
fi

# ────────────────────────────────────────────────────────────────────
# 6. npm dependencies
# ────────────────────────────────────────────────────────────────────
info "Installing npm dependencies..."
(cd "$ROOT" && npm install)

# ────────────────────────────────────────────────────────────────────
# 7. Icons
# ────────────────────────────────────────────────────────────────────
info "Generating application icons..."
bash "$ROOT/scripts/gen-icons.sh"

echo ""
ok "Development environment ready!"
echo ""
echo "  Run the app in dev mode:   make dev"
echo "  Build distributable:       make build"
echo "  Run tests:                 make test"
echo ""
