# MathTalking CSV Editor Desktop

Tauri 2 desktop wrapper for the offline `csv-editor.html` from MathTalking.

## Shape

- Source of truth: `../mathtalking/csv-editor.html`
- Desktop build asset: `src/index.html`
- Native bridge: `src/desktop-bridge.js`
- Bundle ID: `com.mathtalking.csveditor`
- File associations: `.csv` and `.tsv`
- macOS distribution target: Mac App Store
- Windows/Linux distribution target: unsigned direct downloads

## Local Build

```powershell
cargo install tauri-cli --version 2.10.1 --locked
npm run sync
npm run dev
npm run build
```

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
