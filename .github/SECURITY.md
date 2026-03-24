# Security Policy — AegisPDF

## Supported Versions

| Version | Supported |
| ------- | --------- |
| latest  | ✅ Yes     |
| < 0.1.0 | ❌ No      |

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Please report security issues via one of:

1. **GitHub private vulnerability reporting** — use the *Security* tab → *Report a vulnerability*
2. **Email** — security@aegispdf.dev (PGP key available on request)

Include:
- Description of the vulnerability
- Steps to reproduce
- Affected component (Rust core / IPC / frontend / bundler)
- Impact assessment
- Any potential fix if known

We will acknowledge receipt within **48 hours** and aim to triage within **7 days**.

## Security Architecture

AegisPDF is designed with a defence-in-depth model:

| Layer | Control |
|-------|---------|
| IPC | Tauri `invoke` only — no direct DOM → OS access |
| Capabilities | Minimal permissions (`dialog`, `fs`) — no shell/exec/network |
| PDF parsing | `lopdf` in isolated Rust process |
| Annotations | Stored as `.aegis` JSON sidecar, never executed |
| OCR | `tesseract` spawned as a subprocess — input is PNG bytes only |
| No cloud | All processing is 100% offline |

## Automatic Security Checks (CI)

Every push and pull request runs:

- **gitleaks** — hardcoded secret detection
- **cargo audit** — Rust CVE database scan
- **cargo deny** — license + banned crate policy
- **npm audit** — npm CVE scan
- **Trivy** — filesystem secret and misconfiguration scan
- **CodeQL** — static application security testing (JS + Rust)
- **Dependency review** — new dependency CVE/license gate on PRs

## Scope

In-scope:
- Rust backend code execution / memory safety
- IPC command injection
- Path traversal in file operations
- Arbitrary code execution via PDF parsing

Out-of-scope:
- Social engineering
- Vulnerabilities in the OS/WebView2/Tauri runtime itself (report to those projects)
- Self-XSS
