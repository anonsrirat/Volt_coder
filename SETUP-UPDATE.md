# 🔧 Setting up Auto-Update for Volt

Volt checks GitHub Releases on launch and offers a one-click update. Here's how
to wire it up for your own repository.

---

## Step 1 — Push to GitHub

```bash
git init
git add .
git commit -m "⚡ Volt v0.2.0"
git remote add origin https://github.com/<owner>/<repo>.git
git push -u origin main
```

## Step 2 — Point Volt at your repo

Volt reads the repository from (in order):

1. The **build-time env var** `VOLT_GITHUB_REPO=owner/repo`
2. The **fallback** in `src-tauri/src/updater.rs` (`your-org/volt`)

For local builds:

```powershell
$env:VOLT_GITHUB_REPO = "owner/repo"
cd src-tauri
..\ui\node_modules\.bin\tauri build
```

## Step 3 — Publish a release

The included workflow (`.github/workflows/release.yml`) does everything:

```bash
git tag v0.2.0
git push origin v0.2.0
```

GitHub Actions will:
1. Build the app on `windows-latest`
2. Produce `Volt_0.2.0_x64-setup.exe` + `Volt_0.2.0_x64_en-US.msi`
3. Create / update the GitHub Release with those assets

## Step 4 — Verify

Installed copies of Volt will, on next launch:

```
GET https://api.github.com/repos/<owner>/<repo>/releases/latest
```

If `tag_name` (e.g. `v0.2.1`) is newer than the running version, a banner
appears with **Download & install**. Users can also check manually from
**Settings → About**.

---

## How the version check works

`src-tauri/src/updater.rs::is_newer("0.2.1", "0.2.0")` → `true`

- Tags are compared numerically component-by-component (`v` prefix ignored).
- Pre-release suffixes (`0.2.0-beta.1`) are handled gracefully.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| "no releases found" | Repo has no published release | Publish one with a `v*` tag |
| "github returned 403" | Rate-limited (unauthenticated: 60 req/hr) | Wait, or add a token |
| Banner never appears | Running version ≥ latest | Bump the version and re-tag |
| Wrong asset downloaded | No `*-setup.exe` in the release | Ensure the workflow uploaded `nsis` output |

---

## Optional — Cryptographic signing

Volt's updater uses the GitHub Releases API directly (no signature required for
the flow to work). If you want **signed** updates, configure
`tauri-plugin-updater`'s `pubkey` in `tauri.conf.json` and sign with
`TAURI_SIGNING_PRIVATE_KEY` — see `SIGNING.md`.
