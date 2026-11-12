# 🔏 Code Signing Guide — Volt

This guide gets rid of the Windows SmartScreen *"Unknown Publisher"* warning for the
Volt installer and executable.

> [!IMPORTANT]
> Nobody can fully remove the warning **without a real certificate**. Below are the
> three realistic paths, from cheapest to most trustworthy.

---

## Option A — Self-signed certificate (free, for personal/testing only)

✅ No SmartScreen popup **on machines that trust your certificate**
❌ Will still warn on other people's PCs until they install your cert

```powershell
# 1. Create a self-signed code-signing certificate (valid 3 years)
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=Volt Dev" `
  -KeyUsage DigitalSignature `
  -FriendlyName "Volt Self-Signed" `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -NotAfter (Get-Date).AddYears(3)

# 2. Export it (keep the .pfx + password safe)
$pwd = ConvertTo-SecureString -String "change-me" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "volt-selfsigned.pfx" -Password $pwd

# 3. Trust it on THIS machine (removes the warning locally)
Export-Certificate -Cert $cert -FilePath "volt-selfsigned.cer"
Import-Certificate -FilePath "volt-selfsigned.cer" `
  -CertStoreLocation "Cert:\LocalMachine\Root"

# 4. Sign the built artifacts
signtool sign /f volt-selfsigned.pfx /p change-me /fd SHA256 `
  /tr http://timestamp.digicert.com /td SHA256 `
  src-tauri\target\release\bundle\nsis\Volt_0.1.0_x64-setup.exe
```

---

## Option B — OV certificate (paid, ~$200–400/yr)

Buy from **DigiCert / Sectigo / SSL.com**. You get a `.pfx`.

```powershell
signtool sign /f mycert.pfx /p <password> /fd SHA256 `
  /tr http://timestamp.digicert.com /td SHA256 `
  src-tauri\target\release\bundle\nsis\Volt_0.1.0_x64-setup.exe
```

SmartScreen reputation builds up over time (a few hundred downloads).

---

## Option C — EV / Azure Trusted Signing (best, ~$10/mo)

[**Azure Trusted Signing**](https://learn.microsoft.com/azure/trusted-signing/) gives
an EV-equivalent cert with instant SmartScreen trust and no hardware token.

### Tauri native signing config

Add to `src-tauri/tauri.conf.json` → `bundle.windows`:

```jsonc
"windows": {
  "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
  "digestAlgorithm": "sha256",
  "timestampUrl": "http://timestamp.digicert.com",
  "nsis": { "installMode": "currentUser" }
}
```

Then build normally — Tauri signs automatically:

```bash
cd src-tauri
../ui/node_modules/.bin/tauri build
```

> [!TIP]
> Find your thumbprint: `Get-ChildItem Cert:\CurrentUser\My | Select Subject, Thumbprint`

---

## Verifying a signature

```powershell
Get-AuthenticodeSignature "src-tauri\target\release\Volt.exe" | Format-List
```

`Status : Valid` ✅ means Windows now trusts the binary.

---

## Reducing false-positive AV flags (no cert needed)

Tauri apps occasionally trip heuristic scanners. To minimise:

1. **Build release, not debug** (`--release`).
2. Enable the NSIS `installMode: currentUser` (already set — no UAC = fewer flags).
3. Don't pack the installer with UPX.
4. Submit a false-positive report to Microsoft Defender if needed.
