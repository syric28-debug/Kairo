# KAIRO v1.0.0 — Release Notes

> Copy the content below into the GitHub Release description at:
> https://github.com/syric28-debug/Kairo/releases/new

---

# KAIRO v1.0.0

KAIRO — Local Runtime Manager for Windows

## Highlights

KAIRO is a lightweight desktop application that lets you configure, launch, monitor, and manage local background processes — development servers, APIs, workers, and scripts — from a single dashboard, without terminal windows.

- **Generic local service management** — add, edit, and delete any executable, command, or script as a managed service
- **Start / Stop / Restart** with guaranteed full process-tree termination (Windows Job Objects — no orphaned zombie processes)
- **PID tracking & live process monitoring** — real-time PID, CPU usage, memory, and uptime per service
- **Port checking & readiness detection** — distinguishes `Running` (process alive) from `Ready` (port accepting connections) via loopback-only TCP probing
- **Auto Start** — launch KAIRO on Windows boot and auto-start selected services at launch (per-user, no admin required)
- **Auto Restart** — automatic recovery from unexpected crashes with built-in crash-loop protection
- **Windows system tray** — minimize to tray, Start All / Stop All / Restart All from the tray menu; closing the window never kills your services
- **Service logs** — live `stdout` / `stderr` streaming with search, filtering, and per-service 1,000-line circular buffers
- **Persistent configuration & automatic data migration** — atomic, user-space storage in `%APPDATA%`
- **Failure isolation** — a crash in one service never affects any other service

## Downloads

### Windows 64-bit
- `KAIRO-Setup-1.0.0-x64.exe` — for 64-bit Windows (recommended for most modern PCs)

### Windows 32-bit
- `KAIRO-Setup-1.0.0-x86.exe` — for 32-bit Windows (legacy systems)

> **Which one do I need?** **x64** = 64-bit Windows · **x86** = 32-bit Windows. If unsure, open Windows **Settings → System → About** and check the "System type" field.

## Installation

1. Download the installer matching your Windows architecture (see above).
2. Double-click the downloaded `.exe` file.
3. KAIRO installs to your local user directory (`%LOCALAPPDATA%\KAIRO`) — no administrator privileges needed.
4. Launch KAIRO from the Start Menu shortcut and add your first service.

## Known Limitations

- Windows only — KAIRO currently supports Windows 10 and Windows 11 (x64 and x86). No macOS or Linux builds.
- The WebView2 Runtime is required (pre-installed on Windows 10/11).
- Service logs are kept in memory (1,000 lines per service, circular buffer) — they are not written to disk and are cleared when the application exits.
- Port readiness probing covers local TCP ports only (loopback `127.0.0.1` / `[::1]`); it does not verify external network reachability.

---

*Attach exactly these two assets to the release:*
- `KAIRO-Setup-1.0.0-x64.exe`
- `KAIRO-Setup-1.0.0-x86.exe`
