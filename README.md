# miniCSV Desktop

Tauri 2 desktop app for the offline miniCSV editor.

## Downloads

Current unsigned builds from the latest successful GitHub Actions run:

- [macOS universal DMG / app artifact](https://github.com/architectds/minicsv/actions/runs/25644053287/artifacts/6908276154)
- [Windows installer artifact](https://github.com/architectds/minicsv/actions/runs/25644053287/artifacts/6908298149)
- [Linux AppImage / deb artifact](https://github.com/architectds/minicsv/actions/runs/25644053287/artifacts/6908292075)

These are GitHub Actions artifacts from commit `3d2259d`; GitHub expires them
after the artifact retention window. Use the `Desktop Release Builds` workflow
to rebuild fresh packages.

## Shape

- Source of truth: `src/index.html`
- Native bridge: `src/desktop-bridge.js`
- App icon source: `asset/app-icon.png` (cropped from `asset/logo.png`)
- Publisher / signing owner: `Lyncius LLC`
- Bundle ID: `com.lyncius.minicsv`
- File associations: `.csv` and `.tsv`
- File encodings: UTF-8, UTF-8 BOM, UTF-16 LE/BE, Windows-1252, GB18030,
  Big5, Shift_JIS, EUC-KR
- macOS distribution target: Mac App Store
- Windows/macOS/Linux distribution target: unsigned direct downloads

## Local Build

Use Node 18+; CI uses Node 22.

```powershell
npm ci
npm run check
npm run dev
npm run build
```

On Windows, run the build from a Visual Studio x64 developer prompt if Cargo
cannot find the MSVC linker libraries.

`src/index.html` is intentionally checked in as this repo's standalone editor
page. It includes large-dataset virtual scrolling / lazy cell editing updates
plus the native desktop bridge hooks.

## Direct Download Builds

`.github/workflows/release.yml` builds unsigned packages for:

- Windows
- macOS
- Linux

The macOS direct-download artifact is separate from the Mac App Store workflow
below.

## Mac App Store Path

The Mac App Store package must be built on macOS. This repo includes
`.github/workflows/mac-app-store.yml` for a manual GitHub Actions upload.

Required GitHub secrets:

- `APPLE_TEAM_ID`
- `APPLE_APP_CERTIFICATE_BASE64`
- `APPLE_APP_CERTIFICATE_PASSWORD`
- `APPLE_APP_SIGNING_IDENTITY`
- `APPLE_INSTALLER_CERTIFICATE_BASE64`
- `APPLE_INSTALLER_CERTIFICATE_PASSWORD`
- `APPLE_INSTALLER_SIGNING_IDENTITY`
- `MACOS_PROVISION_PROFILE_BASE64`
- `APP_STORE_CONNECT_KEY_ID`
- `APP_STORE_CONNECT_ISSUER_ID`
- `APP_STORE_CONNECT_PRIVATE_KEY`
- `KEYCHAIN_PASSWORD`

The workflow generates `src-tauri/Entitlements.plist`, embeds
`src-tauri/embedded.provisionprofile`, builds a universal macOS app, wraps it
as a signed `.pkg`, and uploads it to App Store Connect.

Use Apple certificates, provisioning profiles, and App Store Connect API keys
from the Lyncius LLC Apple Developer team. The signing identities should appear
as Apple Distribution / Mac Installer Distribution identities for Lyncius LLC.

## Apple Portal Checklist

1. Create App ID `com.lyncius.minicsv` under the Lyncius LLC Apple Developer team.
2. Enable App Sandbox.
3. Create a Mac App Store Connect provisioning profile for that App ID.
4. Export an Apple Distribution or 3rd Party Mac Developer Application `.p12` for Lyncius LLC.
5. Export a Mac Installer Distribution or 3rd Party Mac Developer Installer `.p12` for Lyncius LLC.
6. Create an App Store Connect API key with app upload access.
7. Add the secrets above to the GitHub repo.
8. Run the `Mac App Store Upload` workflow.
