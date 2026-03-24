# AegisPDF Makefile — convenience targets wrapping npm/cargo/tauri.
# Requires: make, Node.js 20+, Rust stable, Tauri CLI.

.PHONY: help setup icons dev build build-linux build-windows \
        build-deb build-rpm build-appimage build-msi build-nsis \
        test lint audit clean distclean

SHELL := /bin/bash
ROOT  := $(dir $(abspath $(lastword $(MAKEFILE_LIST))))

help:                ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) \
	  | awk 'BEGIN{FS=":.*?## "}{printf "  \033[36m%-20s\033[0m %s\n",$$1,$$2}'

# ─────────────────────────────────────────────────────────────────────────────
# Environment setup
# ─────────────────────────────────────────────────────────────────────────────
setup:               ## Full dev environment setup (Linux)
	@chmod +x scripts/setup-dev-linux.sh && ./scripts/setup-dev-linux.sh

icons:               ## Regenerate all app icons from the master SVG
	@chmod +x scripts/gen-icons.sh && ./scripts/gen-icons.sh

# ─────────────────────────────────────────────────────────────────────────────
# Development
# ─────────────────────────────────────────────────────────────────────────────
dev:                 ## Start the Tauri dev server (hot-reload)
	npm run tauri dev

# ─────────────────────────────────────────────────────────────────────────────
# Builds
# ─────────────────────────────────────────────────────────────────────────────
build:               ## Build all targets for the current platform
	npm run tauri build

build-linux:         ## Build Linux bundles (.deb + .rpm + .AppImage)
	npm run tauri build -- --bundles deb,rpm,appimage

build-windows:       ## Build Windows installers (.msi + NSIS .exe)
	npm run tauri build -- --bundles msi,nsis

build-deb:           ## Build Debian package (.deb)
	npm run tauri build -- --bundles deb
	@echo ""
	@echo "──────────────────────────────────────────────────────"
	@find src-tauri/target/release/bundle/deb -name "*.deb" 2>/dev/null | \
	  while read f; do echo "  .deb → $$f"; done
	@echo "Install with:"
	@echo "  sudo dpkg -i <path-to.deb>"
	@echo "──────────────────────────────────────────────────────"

build-rpm:           ## Build RPM package (.rpm)
	npm run tauri build -- --bundles rpm
	@echo ""
	@find src-tauri/target/release/bundle/rpm -name "*.rpm" 2>/dev/null | \
	  while read f; do echo "  .rpm → $$f"; done
	@echo "Install with:  sudo rpm -i <path-to.rpm>"
	@echo "            or sudo dnf install <path-to.rpm>"

build-appimage:      ## Build portable AppImage
	npm run tauri build -- --bundles appimage
	@echo ""
	@find src-tauri/target/release/bundle/appimage -name "*.AppImage" 2>/dev/null | \
	  while read f; do echo "  .AppImage → $$f"; done
	@echo "Run with:  chmod +x <file>.AppImage && ./<file>.AppImage"

build-msi:           ## Build Windows MSI installer (must run on Windows)
	npm run tauri build -- --bundles msi

build-nsis:          ## Build Windows NSIS installer (can cross-compile)
	npm run tauri build -- --bundles nsis

# ─────────────────────────────────────────────────────────────────────────────
# Testing
# ─────────────────────────────────────────────────────────────────────────────
test:                ## Run Rust unit + integration tests
	cargo test --manifest-path src-tauri/Cargo.toml

lint:                ## Run clippy + rustfmt check + ESLint + tsc
	cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
	cargo fmt --manifest-path src-tauri/Cargo.toml --check
	npm run lint -- --max-warnings 0
	npm run type-check

audit:               ## Run all security scans locally
	@echo "==> cargo audit (CVE database)"
	cargo audit --manifest-path src-tauri/Cargo.toml --deny warnings --deny unsound --deny yanked
	@echo "==> cargo deny (license + ban + source policy)"
	cargo deny --manifest-path src-tauri/Cargo.toml check
	@echo "==> npm audit"
	npm audit --audit-level=high
	@echo "==> gitleaks"
	gitleaks detect --source . --config .gitleaks.toml || echo "[WARN] gitleaks not installed"

# ─────────────────────────────────────────────────────────────────────────────
# Clean
# ─────────────────────────────────────────────────────────────────────────────
clean:               ## Remove compiled artifacts (keeps node_modules)
	cargo clean --manifest-path src-tauri/Cargo.toml
	rm -rf dist

distclean: clean     ## Remove compiled artifacts AND node_modules
	rm -rf node_modules
