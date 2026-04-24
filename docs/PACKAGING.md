# Packaging

## Windows (.msi)

From a developer PowerShell / CMD:

```bash
npm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/msi/`. The build also
produces an `nsis/` installer if configured.

### Signing (optional, recommended)

Get a code-signing certificate (EV or OV). Configure `tauri.conf.json`:

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_THUMBPRINT_HERE",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

Unsigned builds still run; Windows SmartScreen will warn on first launch
until the signature accumulates reputation.

## macOS (.dmg)

```bash
npm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/dmg/` and
`.../macos/Job Search Tool.app`.

### Signing + notarization (optional, recommended)

Set environment variables before building:

```bash
export APPLE_ID="your.apple.id@example.com"
export APPLE_ID_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="XXXXXXXXXX"
export APPLE_CERTIFICATE="Developer ID Application: Your Name (XXXXXXXXXX)"
```

Tauri automatically signs with the identity and submits for notarization
when these are present.

Unsigned builds run but Gatekeeper will quarantine the `.app` on first
launch (`System Preferences → Privacy & Security → Open anyway`).

## Distribution

This is a single-user tool, so typical distribution is "build on the
machine that runs it". If you want to distribute:

- Windows: `Job Search Tool_0.1.0_x64_en-US.msi`
- macOS: `Job Search Tool_0.1.0_aarch64.dmg` (Apple Silicon) or `_x64.dmg`
  (Intel) depending on the build host.

Cross-compiling macOS from Windows is not supported; build on each OS.

## Telemetry & network

- No analytics, no crash reporting, no auto-update service.
- Outbound network is limited to the ATS domains listed in
  `tauri.conf.json → app.security.csp` and whatever URL the user hands to
  the LinkedIn / URL import flow at runtime.
