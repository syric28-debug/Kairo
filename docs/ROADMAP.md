# KAIRO — 12-Phase Roadmap

This document outlines the evolutionary development roadmap for **KAIRO** (Local Runtime Manager for Windows), tracking milestones from initial scaffolding to enterprise-grade distribution.

All phases permanently uphold the **Black + White + Transparent Glass** visual identity.

---

## Roadmap Overview

| Phase | Milestone | Focus Area | Status |
|---|---|---|---|
| **Phase 0** | **Project Initialization** | Tauri 2, React 19, TypeScript, Rust foundation, Glass tokens, Docs | **Completed** |
| **Phase 1** | **Basic Desktop UI** | Glass frame, sidebar, empty state, service list view skeleton | **Completed** |
| **Phase 2** | **Add Service + Persistence** | Service configuration modal, JSON storage repository, validation | **Completed** |
| **Phase 3** | **Start / Stop / Restart** | Child process spawning, Windows Job Objects, clean termination | **Completed** |
| **Phase 4** | **Process Monitoring** | Real-time PID, CPU usage %, memory working set, uptime ticker | **Completed** |
| **Phase 5** | **Port Checking** | TCP socket probing, port collision alerts, HTTP health pings | **Completed** |
| **Phase 6** | **Windows Auto-Start** | Windows Registry Run key (`HKCU\Run`), boot-time launching | **Completed** |
| **Phase 7** | **Background System Tray** | Minimize to tray, tray context menu (Start All/Stop All/Quit) | **Completed** |
| **Phase 8** | **Auto Restart** | Crash detection, exponential backoff, retry limiters | **Completed** |
| **Phase 9** | **Logs** | Real-time stdout/stderr streaming, search/filter, log export | **Completed** |
| **Phase 10** | **Production Packaging & Branding** | KAIRO rebranding, icons, NSIS x64/x86 installers | **Completed** |
| **Phase 11** | **GitHub Release** | CI/CD GitHub Actions workflow, multi-artifact release builds | Planned |
| **Phase 12** | **Auto Updater** | Tauri updater plugin, cryptographic signature verification | Planned |

---

## Detailed Phase Specifications

### Phase 0: Project Initialization (Current)
- **Scope**:
  - Environment inspection (Node, npm, Rust, Cargo, Tauri).
  - Clean project scaffolding without legacy dependencies.
  - Setup Vite + React 19 + TypeScript + Tauri 2 configuration.
  - Establish Rust core modules (`commands`, `models`, `services`, `repository`, `utils`, `error`).
  - Implement permanent Black + White + Transparent Glass design system and design tokens.
  - Deploy minimal Phase 0 boot screen.
  - Provide full architecture (`ARCHITECTURE.md`) and roadmap (`ROADMAP.md`) specifications.
  - Verify compile checks and desktop window launch.

---

### Phase 1: Basic Desktop UI
- **Scope**:
  - Full desktop layout: custom glass titlebar, navigation sidebar, and main view container.
  - Service card component with monochrome status indicators.
  - Empty state presentation with glass styling and guidance for adding the first service.
  - Search bar and filtering controls (All, Running, Stopped).
  - Responsive glassmorphism panels with backdrop blur and subtle hover states.

---

### Phase 2: Add Service + Persistence
- **Scope**:
  - "Add Service" modal dialog with translucent backdrop blur.
  - Generic form inputs: Name, Executable Path (with Windows file picker), Arguments, Working Directory, Custom Environment Variables, Target Port, Auto-Start checkbox, Auto-Restart checkbox.
  - Rust configuration repository: atomic JSON persistence in `%APPDATA%/LocalServiceManager/services.json`.
  - Input validation, path verification, and collision avoidance.

---

### Phase 3: Start / Stop / Restart
- **Scope**:
  - Process execution engine using Rust `std::process::Command`.
  - Windows Job Object attachment (`CreateJobObjectW`, `AssignProcessToJobObject`) with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` to guarantee 100% kill-tree accuracy.
  - `CREATE_NO_WINDOW` flag to prevent terminal window popups.
  - Asynchronous IPC commands: `start_service`, `stop_service`, `restart_service`.
  - Deterministic state machine transitions with UI feedback.

---

### Phase 4: Process Monitoring
- **Scope**:
  - Background telemetry loop in Rust querying process metrics via Windows API / `sysinfo`.
  - Track metrics: PID, User/Kernel CPU %, Private Working Set Memory (MB/GB), Uptime.
  - Stream periodic updates to the UI via Tauri IPC events.
  - Monochrome sparklines / metric counters on service cards.

---

### Phase 5: Port Checking
- **Scope**:
  - Asynchronous TCP port socket tester (`std::net::TcpStream`).
  - Active detection of whether the service's designated port is listening.
  - Port collision pre-flight check: alert user if a port is already occupied before launching.
  - Real-time port status badge on service cards (e.g., `:8080 Active`).

---

### Phase 6: Windows Auto-Start
- **Scope**:
  - Integration with Windows Startup registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`).
  - Option to launch Local Service Manager on Windows logon.
  - Option to launch configured auto-start services on app startup.
  - Non-intrusive background boot mode (launch minimized).

---

### Phase 7: Background System Tray
- **Scope**:
  - Tauri system tray integration with native dark icon.
  - Minimize-to-tray on window close (`x`) and minimize (`_`) button actions.
  - Tray context menu:
    - Open Window
    - Start All Services
    - Stop All Services
    - Separator
    - Quit Local Service Manager
  - Native Windows balloon / toast notifications on service failure.

---

### Phase 8: Auto Restart
- **Scope**:
  - Process exit watcher monitoring child process exit handles.
  - Automatic restart trigger for services flagged with `auto_restart: true`.
  - Crash loop protection: exponential backoff delays and maximum retry thresholds (e.g., max 5 restarts within 60 seconds).
  - Warning indicators for unstable services.

---

### Phase 9: Logs
- **Scope**:
  - Asynchronous stdout and stderr capture pipes on spawned processes.
  - In-memory circular log buffer per service (e.g., last 1,000 lines).
  - Real-time log viewer drawer with monospace typography, pause stream, text filter, and "Clear" / "Export to File" capabilities.
  - Persistent log rotation to disk (`%APPDATA%/LocalServiceManager/logs/`).

---

### Phase 10: Windows .exe Packaging
- **Scope**:
  - Tauri bundler configuration for Windows targets.
  - Production build optimization (LTO, strip debug symbols, minified bundle).
  - Standalone NSIS installer generation (`.exe`) and optional MSI installer.
  - Application icon stamping and Windows Authenticode signing hooks.

---

### Phase 11: GitHub Release
- **Scope**:
  - GitHub Actions CI/CD matrix workflow (`build-release.yml`).
  - Automated building on `windows-latest`.
  - Automatic release drafting, semantic changelog generation, and asset attachment.

---

### Phase 12: Auto Updater
- **Scope**:
  - Tauri v2 official updater plugin integration (`tauri-plugin-updater`).
  - Ed25519 public/private key verification for update payloads.
  - In-app update notification modal with "Install & Relaunch" workflow.
