# MathTalking CSV Editor Desktop

Tauri 2 desktop app for the offline MathTalking CSV editor.

## Downloads

Current unsigned builds from the latest successful GitHub Actions run:

- [macOS universal DMG / app artifact](https://github.com/architectds/minicsv/actions/runs/25637940166/artifacts/6906433438)
- [Windows installer artifact](https://github.com/architectds/minicsv/actions/runs/25637940166/artifacts/6906442680)
- [Linux AppImage / deb artifact](https://github.com/architectds/minicsv/actions/runs/25637940166/artifacts/6906442434)

These are GitHub Actions artifacts from commit `250d1d5`; GitHub expires them
after the artifact retention window. Use the `Desktop Release Builds` workflow
to rebuild fresh packages.

## Shape

- Source of truth: `src/index.html`
- Native bridge: `src/desktop-bridge.js`
- App icon source: `asset/app-icon.png` (cropped from `asset/logo.png`)
- Bundle ID: `com.mathtalking.csveditor`
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
page. It was copied from the latest MathTalking `csv-editor.html` and includes
the large-dataset virtual scrolling / lazy cell editing updates plus the native
desktop bridge hooks.

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

## Apple Portal Checklist

1. Create App ID `com.mathtalking.csveditor`.
2. Enable App Sandbox.
3. Create a Mac App Store Connect provisioning profile for that App ID.
4. Export an Apple Distribution or 3rd Party Mac Developer Application `.p12`.
5. Export a Mac Installer Distribution or 3rd Party Mac Developer Installer `.p12`.
6. Create an App Store Connect API key with app upload access.
7. Add the secrets above to the GitHub repo.
8. Run the `Mac App Store Upload` workflow.
