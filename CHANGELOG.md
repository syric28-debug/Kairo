# Changelog

All notable changes to the KAIRO project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.0.1] — First Signed Updater Release

**Release tag:** `v1.0.1`

The first KAIRO release published with signed automatic-update artifacts.

### Added — Automatic Update System (Phase 12)

- **Official GitHub Releases update channel:** KAIRO now checks
  `https://github.com/syric28-debug/Kairo/releases/latest/download/latest.json`
  (HTTPS only) for newer stable releases using the official Tauri 2 updater
  plugin. No third-party update servers, mirrors, or arbitrary download URLs.
- **Semantic version comparison:** Update availability uses proper semantic
  ordering (`1.0.0` < `1.1.0` < `1.10.0` < `2.0.0`) — never plain string
  comparison. Covered by unit tests including malformed-version handling.
- **Settings → Updates section:** Current version, update status, manual
  "Check for Updates" button, and an optional "Check for updates on startup"
  toggle (persisted in `settings.json`, backward compatible via serde default).
- **Non-intrusive update notification:** A dismissible banner
  ("KAIRO X.Y.Z is available.") with Install Update / Later actions. Never
  forced; also visible when reopening KAIRO from the system tray.
- **Release-notes preview:** The update dialog shows the real release notes
  from the official GitHub release before installation. If notes cannot be
  retrieved, only version information is shown.
- **Signature-verified installation:** Update packages are verified with the
  configured Tauri updater minisign signature before installation. Unsigned or
  invalid packages are rejected. No shell execution, no download-and-execute.
- **Architecture-correct updates:** The running binary's architecture maps to
  `windows-x86_64` / `windows-i686` platform keys — an x86 installation can
  never receive an x64 update, and vice versa. Covered by unit tests.
- **Safe update installation:** Running managed services are stopped through
  the existing exact-PID-based lifecycle before the update is applied; the NSIS
  installer runs in passive mode and restarts the new version automatically.
- **Failure resilience:** Network failures, unreachable GitHub, missing
  releases, invalid metadata, and interrupted/corrupted downloads all leave
  KAIRO fully usable with clear status text ("Unable to check for updates.")
  — no error popups and no broken half-updated states.
- **Update source validation:** Endpoint validation helpers enforce HTTPS and
  the official GitHub host; updater state (`get_updater_status`) reports the
  installed version, platform target, and signing configuration honestly.
- **Production release process:** Documented in `docs/UPDATES.md` (signing
  setup, dual-architecture builds with updater artifacts, `latest.json`
  manifest format, draft → verify → publish workflow).

> Note: `v1.0.1` is the first release that enables automatic updates. The
> previous `v1.0.0` release predates the updater and contains no `latest.json`
> or signature files; existing v1.0.0 installations continue to work normally
> and will be able to update through GitHub Releases once `v1.0.1` is
> published.

### Security

- Updater configured for HTTPS-only transport to the official GitHub host.
- Signing public key location prepared in `tauri.conf.json`; production
  private key handling documented — private keys are never stored in the
  repository, source, or configuration files.

---

## [1.0.0] — Initial Release

**Release date:** September 2026
**Release tag:** [`v1.0.0`](https://github.com/syric28-debug/Kairo/releases/tag/v1.0.0)

KAIRO — Local Runtime Manager for Windows. First official production release.

### Added — Service Management
- **Generic local service management:** Register any executable, command, or script as a managed local service (development servers, APIs, background workers, CLI tools).
- **Add / Edit / Delete services** via the Add Generic Service modal, with full field-level configuration (name, description, executable, arguments, working directory, environment variables, target port).
- **Start / Stop / Restart** controls for each service, with full process-tree termination via Windows Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`).
- **PID tracking:** Live Windows Process ID display for every running service.
- **Process monitoring:** Real-time CPU usage, private working set memory, and uptime counters with per-second background polling.

### Added — Port & Readiness
- **Port checking:** Optional TCP target port per service (1–65535).
- **Port readiness detection:** Loopback-only TCP probing (`127.0.0.1` / `[::1]`) that distinguishes between `Running` (process alive) and `Ready` (port actively accepting connections), including a `Waiting...` transitional state and stale-probe rejection.

### Added — Startup & Reliability
- **Windows application Auto Start:** Optional per-user launch-on-Windows-boot registration via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` (no administrator elevation required).
- **Service Auto Start:** Per-service auto-start on application launch with a global master switch in Settings.
- **Auto Restart:** Automatic recovery of unexpectedly terminated processes, with intentional-stop protection and crash-loop protection (maximum 3 restart attempts within 30 seconds, suspended to `Failed` if exceeded).
- **Failure isolation:** Every service runs in complete isolation; one service's crash never affects any other running service.

### Added — Application Experience
- **Windows system tray integration:** Minimize-to-tray on window close, plus tray menu with Open Dashboard, Start All, Stop All, Restart All, and Exit.
- **Service logs:** Asynchronous capture of `stdout` and `stderr` into a 1,000-line circular in-memory buffer per service, with a centralized Logs view offering stream tabs, search, clear, copy, and smart auto-scroll.
- **Persistent configuration:** Atomic write-and-rename storage of service definitions and settings in `%APPDATA%\com.kairo.localruntimemanager\`.
- **Automatic data migration:** Seamless migration of legacy configuration storage locations without user intervention.

### Added — Branding & Distribution
- **KAIRO branding:** Custom application logo, tray icon, window icon, and installer icon across the application and installers.
- **x64 installer:** `KAIRO-Setup-1.0.0-x64.exe` for 64-bit Windows (built with the `x86_64-pc-windows-msvc` target).
- **x86 installer:** `KAIRO-Setup-1.0.0-x86.exe` for 32-bit Windows (built with the `i686-pc-windows-msvc` target).
- **Installer experience:** Per-user NSIS installation to `%LOCALAPPDATA%\KAIRO` with Start Menu shortcut and clean uninstaller — no administrator privileges required.

### Technology
- Desktop engine: Tauri 2 · Backend: Rust (2021 edition) · Frontend: React 19 + TypeScript · Bundler: Vite
