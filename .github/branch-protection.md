# Branch Protection Setup Guide

After pushing to GitHub, apply these settings in
**Repository → Settings → Branches → Add branch ruleset**.

## `main` and `dev` branches

### 1. Required status checks (block merge until all pass)

Enable "Require status checks to pass before merging" and add these checks:

| Check name | Workflow |
|------------|----------|
| `🚦 Security gate` | ci.yml |
| `✅ Quality gate` | ci.yml |
| `🔑 Secret scan (gitleaks)` | ci.yml |
| `📦 Rust supply-chain (audit + deny)` | ci.yml |
| `📦 npm supply-chain (audit + licenses)` | ci.yml |
| `🛡️  Trivy (secrets + misconfig)` | ci.yml |
| `🦀 Rust format (rustfmt)` | ci.yml |
| `🦀 Rust lint (clippy strict)` | ci.yml |
| `🦀 Rust tests + coverage` | ci.yml |
| `⚛️  Frontend (tsc strict + ESLint)` | ci.yml |
| `🔍 CodeQL — JavaScript/TypeScript` | codeql.yml |

### 2. Pull request requirements

- ✅ Require a pull request before merging
- ✅ Require at least **1 approval** from CODEOWNERS
- ✅ Dismiss stale pull request approvals when new commits are pushed
- ✅ Require review from Code Owners (uses `.github/CODEOWNERS`)
- ✅ Require conversation resolution before merging

### 3. Push rules

- ✅ Restrict pushes that create matching branches — only allow org members
- ✅ Block force pushes
- ✅ Block deletions of the `main` branch

### 4. Merge queue (recommended for high-traffic repos)

Enable the merge queue to serialize merges and prevent race conditions on status checks.

---

## GitHub Actions secret requirements

Add these repository secrets before the release workflow will function:

| Secret | Required for | How to generate |
|--------|-------------|-----------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Signed updater artifacts | `npm run tauri signer generate -- -w .tauri/key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Signed updater artifacts | Chosen at key generation |
| `GITLEAKS_LICENSE` | Gitleaks paid tier (optional) | https://gitleaks.io |

---

## Rulesets CLI (alternative to UI)

```bash
gh api repos/{owner}/{repo}/rulesets \
  --method POST \
  --input .github/ruleset.json
```

(A `ruleset.json` template can be generated from the GitHub UI after initial setup.)
