#!/usr/bin/env bash
# install.sh — AegisPDF installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/aegispdf/aegispdf/main/install.sh | sudo bash
#
# What it does:
#   Debian/Ubuntu  — adds the signed APT repository and runs apt-get install
#   Fedora/RHEL    — downloads the latest .rpm from GitHub Releases
#   Other Linux    — downloads the latest .AppImage from GitHub Releases
#   macOS / other  — prints a helpful error

set -euo pipefail

# ── constants ────────────────────────────────────────────────────────────────

GITHUB_OWNER="aegispdf"
GITHUB_REPO="aegispdf"
PACKAGE_NAME="aegispdf"
PAGES_BASE="https://${GITHUB_OWNER}.github.io/${GITHUB_REPO}"
KEYRING_PATH="/usr/share/keyrings/${PACKAGE_NAME}.gpg"
APT_SOURCE_PATH="/etc/apt/sources.list.d/${PACKAGE_NAME}.list"
GITHUB_API="https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest"

# ── helpers ──────────────────────────────────────────────────────────────────

info() { printf '\033[0;32m→\033[0m %s\n' "$*"; }
warn() { printf '\033[0;33m⚠\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[0;31m✗\033[0m %s\n' "$*" >&2; exit 1; }

need_root() {
  [[ "${EUID:-$(id -u)}" -eq 0 ]] || die "This installer must be run as root. Try: sudo bash install.sh"
}

require_cmd() {
  command -v "$1" &>/dev/null || die "Required command not found: $1 — please install it and retry."
}

is_debian_based() {
  [[ "${DISTRO_ID:-}" == "ubuntu"  ||
     "${DISTRO_ID:-}" == "debian"  ||
     "${DISTRO_LIKE:-}" == *"debian"* ||
     "${DISTRO_LIKE:-}" == *"ubuntu"* ]]
}

is_rpm_based() {
  [[ "${DISTRO_ID:-}" == "fedora"     ||
     "${DISTRO_ID:-}" == "rhel"       ||
     "${DISTRO_ID:-}" == "centos"     ||
     "${DISTRO_ID:-}" == "almalinux"  ||
     "${DISTRO_ID:-}" == "rocky"      ||
     "${DISTRO_ID:-}" =~ "opensuse"   ||
     "${DISTRO_LIKE:-}" == *"fedora"* ||
     "${DISTRO_LIKE:-}" == *"rhel"*   ]]
}

latest_release_url() {
  # $1 — grep pattern for the asset filename
  require_cmd curl
  curl -fsSL "$GITHUB_API" \
    | grep '"browser_download_url"' \
    | grep -m1 "$1" \
    | cut -d '"' -f 4
}

# ── distro detection ─────────────────────────────────────────────────────────

DISTRO_ID=""
DISTRO_LIKE=""
if [[ -f /etc/os-release ]]; then
  # shellcheck source=/dev/null
  . /etc/os-release
  DISTRO_ID="${ID:-}"
  DISTRO_LIKE="${ID_LIKE:-}"
fi

# ── dispatch ─────────────────────────────────────────────────────────────────

need_root

if [[ "$(uname -s)" != "Linux" ]]; then
  die "Unsupported OS: $(uname -s). Download the installer from https://github.com/${GITHUB_OWNER}/${GITHUB_REPO}/releases"
fi

# ─── Debian / Ubuntu ─────────────────────────────────────────────────────────
if is_debian_based; then
  info "Detected Debian/Ubuntu-based system — installing via APT repository"

  require_cmd apt-get

  # Ensure dependencies are present
  apt-get install -y --no-install-recommends curl gpg 2>/dev/null

  # Import signing key
  info "Adding AegisPDF signing key → ${KEYRING_PATH}"
  curl -fsSL "${PAGES_BASE}/aegispdf.gpg" \
    | gpg --dearmor -o "${KEYRING_PATH}"
  chmod 644 "${KEYRING_PATH}"

  # Add repository
  info "Adding APT source → ${APT_SOURCE_PATH}"
  printf 'deb [arch=amd64 signed-by=%s] %s/apt stable main\n' \
    "${KEYRING_PATH}" "${PAGES_BASE}" \
    > "${APT_SOURCE_PATH}"

  # Install
  info "Running apt-get update && apt-get install ${PACKAGE_NAME}"
  apt-get update -qq
  apt-get install -y "${PACKAGE_NAME}"

  info "Done! Launch AegisPDF from your application menu or by running: aegispdf"

# ─── Fedora / RHEL / CentOS / openSUSE ───────────────────────────────────────
elif is_rpm_based; then
  info "Detected RPM-based system — downloading latest .rpm from GitHub Releases"

  RPM_URL="$(latest_release_url '\.rpm$')"
  [[ -n "$RPM_URL" ]] || die "Could not find an .rpm asset in the latest GitHub Release."

  TMP_RPM="$(mktemp /tmp/aegispdf-XXXXXX.rpm)"
  info "Downloading ${RPM_URL}"
  curl -fsSL -o "$TMP_RPM" "$RPM_URL"

  if command -v dnf &>/dev/null; then
    dnf install -y "$TMP_RPM"
  elif command -v rpm &>/dev/null; then
    rpm -ivh "$TMP_RPM"
  else
    die "Neither 'dnf' nor 'rpm' found — cannot install the .rpm package."
  fi
  rm -f "$TMP_RPM"

  info "Done! Launch AegisPDF from your application menu or by running: aegispdf"

# ─── Other Linux (AppImage fallback) ─────────────────────────────────────────
else
  warn "Unrecognised distribution: ${DISTRO_ID:-unknown}"
  info "Falling back to AppImage installation"

  APPIMAGE_URL="$(latest_release_url '\.AppImage$')"
  [[ -n "$APPIMAGE_URL" ]] || die "Could not find an .AppImage asset in the latest GitHub Release."

  INSTALL_DIR="/opt/aegispdf"
  mkdir -p "$INSTALL_DIR"
  DEST="${INSTALL_DIR}/aegispdf.AppImage"

  info "Downloading ${APPIMAGE_URL}"
  curl -fsSL -o "$DEST" "$APPIMAGE_URL"
  chmod +x "$DEST"

  # Create a wrapper in PATH
  ln -sf "$DEST" /usr/local/bin/aegispdf

  # Desktop entry
  cat > /usr/share/applications/aegispdf.desktop << 'EOF'
[Desktop Entry]
Name=AegisPDF
Exec=/usr/local/bin/aegispdf
Icon=aegispdf
Type=Application
Categories=Utility;
Comment=Privacy-focused offline PDF workspace
EOF

  info "Done! AppImage installed to ${DEST}"
  info "Run: aegispdf  or find it in your application menu."
fi
