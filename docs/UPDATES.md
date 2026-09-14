# KAIRO — Updates & Production Release Process (Phase 12)

This document describes how KAIRO's automatic update system works and the exact,
safe procedure for publishing a new release so that existing installations can
update themselves.

- **Update mechanism:** Official [Tauri 2 updater plugin](https://v2.tauri.app/plugin/updater/)
- **Update channel:** Stable GitHub Releases only
- **Update source (pinned in `src-tauri/tauri.conf.json`):**
  `https://github.com/syric28-debug/Kairo/releases/latest/download/latest.json`
- **Architectures:** `windows-x86_64` (x64) and `windows-i686` (x86) — selected
  automatically per running installation
- **Identifier:** `com.kairo.localruntimemanager`

---

## 1. How the updater works at runtime

1. On startup (optionally, and always on demand from **Settings → Updates**),
   KAIRO's Rust backend asks the updater plugin to fetch `latest.json` from the
   official GitHub Releases channel over HTTPS.
2. The plugin compares the released semver against the installed version
   (real semantic comparison — `1.10.0` > `1.9.0`).
3. If a newer stable version exists, the UI shows a non-intrusive banner and a
   release-notes preview. Nothing is downloaded or installed without an explicit
   user action.
4. On **Install Update**:
   - any running managed services are stopped cleanly via the existing
     PID-based `stop_service` lifecycle (no name-based process killing);
   - the installer package for the **running architecture** is downloaded with
     progress feedback;
   - the package signature is verified against the configured minisign public
     key — an invalid/missing signature aborts the install;
   - the verified NSIS installer is launched in passive mode; the app exits and
     the installer restarts the new version automatically.
5. Failure at any step leaves the current installation untouched and KAIRO fully
   usable. User data in `%APPDATA%\com.kairo.localruntimemanager` is never
   modified by an update.

---

## 2. One-time production signing setup (required before the first updated release)

**This is the only manual step that requires a real secret. It must be done by
the maintainer on a trusted machine. Never share or commit the private key.**

1. Generate the Tauri updater keypair:

   ```bash
   npx tauri signer generate -w "%USERPROFILE%\.tauri\kairo.key"
   ```

   You will be prompted for a password protecting the private key.

2. This produces:
   - `%USERPROFILE%\.tauri\kairo.key` — **private key (secret, never commit)**
   - `%USERPROFILE%\.tauri\kairo.key.pub` — public key (safe to embed)

3. Open the public key file and copy its **entire base64 contents** (it starts
   with `dW50cnVzdGVkIGNvbW1lbnQ6...`) into
   `src-tauri/tauri.conf.json`:

   ```json
   "plugins": {
     "updater": {
       "pubkey": "<paste the full public key string here>",
       ...
     }
   }
   ```

   Until this is done, production builds report *"Updates are not configured in
   this build"* and update checks are intentionally unavailable — this is the
   expected, safe behavior for development builds.

4. Store the private key password safely (e.g. your password manager).

**Pre-flight: verify that `tauri.conf.json`'s `pubkey` actually belongs to the
private key you hold** (do this once, before the first signed release):

The `pubkey` shipped in `tauri.conf.json` and the private key must be one
keypair. Tauri's signer writes both files from the same random seed, so they
always correspond. A key file whose comment key-ID does **not** equal its
embedded key-ID, or a pubkey that was copied from somewhere other than the
`kairo.key.pub` file generated together with `kairo.key`, will produce updates
that **fail cryptographic verification** at install time.

Check with the real tooling (it requires your private key password — this is
expected and safe, and the password is never written anywhere):

```bash
# 1. Sign a throwaway file with YOUR private key and YOUR password:
npx tauri signer sign -f "%USERPROFILE%\.tauri\kairo.key" -p "<your password>" "%TEMP%\sign-test.txt"

# 2. The returned signature file (".sig") embeds the key ID. The hex key ID in
#    its untrusted comment must equal the hex key ID in kairo.key.pub's
#    comment. minisign verification enforces this equality: if the pubkey and
#    the private key do not belong to the same keypair, the signature can never
#    verify and updates will always be rejected at install time.
type "%TEMP%\sign-test.txt.sig"
type "%USERPROFILE%\.tauri\kairo.key.pub"
```

If you are not 100% sure that the embedded pubkey corresponds to a private key
you control, **generate a fresh keypair** (step 1) and replace the pubkey in
`tauri.conf.json`. Never release a signed update against a pubkey you cannot
generate a matching signature for.

**Security rules (non-negotiable):**

- The private key must never be committed to Git, embedded in source, or placed
  in `tauri.conf.json`, README, CI logs, or chat.
- Set the signing environment variables only in the build shell / GitHub
  Actions secrets (see below), never in any tracked file.


---

## 3. Production release process (for every future version)

Follow these steps in order. Example: releasing `v1.1.0` while `v1.0.0` is
current.

### Step 1 — Update the version

Set the same version in all three places:

- `package.json` → `"version": "1.1.0"`
- `src-tauri/Cargo.toml` → `version = "1.1.0"`
- `src-tauri/tauri.conf.json` → `"version": "1.1.0"`

Then run `cargo check` once to refresh `Cargo.lock`.

### Step 2 — Verify the codebase

```bash
cargo test
cargo check --all-targets
npx tsc --noEmit
npm run build
```

All four must pass. Update `CHANGELOG.md` with the new version's notes.

### Step 3 — Set signing environment variables (build shell only)

In the terminal that will run the release builds:

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content "$env:USERPROFILE\.tauri\kairo.key" -Raw)
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "<the private key password>"
```

(Or configure `TAURI_SIGNING_PRIVATE_KEY` /
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` as GitHub Actions secrets if releases are
built in CI.)

### Step 4 — Build both architectures **with updater artifacts enabled**

```bash
# x64
npx tauri build --target x86_64-pc-windows-msvc --config src-tauri/tauri.release.conf.json

# x86
rustup target add i686-pc-windows-msvc
npx tauri build --target i686-pc-windows-msvc --config src-tauri/tauri.release.conf.json
```

The extra `--config src-tauri/tauri.release.conf.json` enables
`bundle.createUpdaterArtifacts`, which produces the installer packages together
with their required `.sig` updater signatures.

**If `.sig` files are not produced, STOP — do not publish.** Unsigned releases
cannot be verified and the updater will (correctly) reject them.


### Step 5 — Create the GitHub Release (draft first)

1. On GitHub, create a **new release** with tag `v1.1.0` (target `main`).
   Do **not** publish it yet.
2. Rename the built installers to match the established asset naming:

   - `KAIRO-Setup-1.1.0-x64.exe`
   - `KAIRO-Setup-1.1.0-x86.exe`
   - plus each matching `.sig` file produced by the build

3. Prepare the official update manifest **`latest.json`** for the release:

   ```json
   {
     "version": "1.1.0",
     "notes": "Concise release notes shown inside KAIRO before updating.",
     "pub_date": "2026-10-01T12:00:00Z",
     "platforms": {
       "windows-x86_64": {
         "signature": "<contents of KAIRO-Setup-1.1.0-x64.exe.sig>",
         "url": "https://github.com/syric28-debug/Kairo/releases/download/v1.1.0/KAIRO-Setup-1.1.0-x64.exe"
       },
       "windows-i686": {
         "signature": "<contents of KAIRO-Setup-1.1.0-x86.exe.sig>",
         "url": "https://github.com/syric28-debug/Kairo/releases/download/v1.1.0/KAIRO-Setup-1.1.0-x86.exe"
       }
     }
   }
   ```

   Rules for `latest.json`:

   - `"version"` **must not** carry a `v` prefix and must be valid semver.
   - `"signature"` is the full contents of the corresponding `.sig` file.
   - `"url"` values must point at the official GitHub release download URLs
     (GitHub redirects to its CDN automatically; do not use any other host).
   - Both platform keys are mandatory while KAIRO ships x64 **and** x86 builds.
     A missing platform key simply means that architecture cannot update (the
     updater reports "target not found" — safe, but incomplete).

4. Upload `latest.json` as a release asset alongside the installers.

### Step 6 — Verify release metadata, then publish

Before clicking **Publish**, verify:

- [ ] Tag is `v1.1.0`; release is stable (not a prerelease/draft)
- [ ] `latest.json` `version` matches the tag and the built app version
- [ ] Both `signature` values match their `.sig` files exactly
- [ ] Both `url` values download correctly (test in a browser)
- [ ] Real release notes pasted into the GitHub release body
- [ ] `package.json`, `Cargo.toml`, and `tauri.conf.json` versions were bumped together

Then publish the release. Publishing makes `latest.json` instantly available at
the pinned update endpoint for all existing installations.

### Step 7 — Post-release verification

On a test machine with KAIRO `v1.0.0` installed (never the production machine
first):

1. Open **Settings → Updates** → **Check for Updates**
   → must show *"KAIRO 1.1.0 is available."*
2. Verify the release-notes preview matches the official notes.
3. Install on a test x64 machine and a test x86 machine. Confirm the new
   version launches and **all services, settings, and logs survived**.
4. Repeat a check with the network disconnected → must report
   *"Unable to check for updates."* and KAIRO must keep working.
5. Finalize `CHANGELOG.md` release links if needed.

---

## 4. Development vs production builds

| | Development (`npm run tauri dev`) | Production release build |
|---|---|---|
| Signing key required | No | Yes (`TAURI_SIGNING_PRIVATE_KEY`) |
| Updater artifacts | Not created | Created (`--config src-tauri/tauri.release.conf.json`) |
| Update checks | Report *"Updates are not configured in this build"* unless a real pubkey is configured | Fully active once the pubkey is set |

`npm run build`, `cargo check`, and `cargo test` never require signing secrets.

---

## 5. Safety guarantees recap

- HTTPS-only transport to the official GitHub host (validated in code and tests)
- Signature verification before installation (no bypass path exists in the app)
- No shell execution, no PowerShell/cmd download-and-execute, no arbitrary URLs
- No weakening of Windows security features
- Architecture-correct updates (`windows-x86_64` / `windows-i686`)
- User data and service configuration untouched by updates
- Running managed services stopped via the existing PID-based safety model
- Every failure mode leaves KAIRO running and usable
