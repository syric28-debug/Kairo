# KAIRO

KAIRO — Local Runtime Manager for Windows

---

## Overview

**KAIRO** is a modern, lightweight, and robust desktop runtime manager engineered specifically for Microsoft Windows. It provides a centralized, hardware-accelerated control center to configure, launch, monitor, and manage local background processes, development servers, microservices, AI inference engines, and command-line workers without cluttering your desktop with terminal windows.

### What Problem Does KAIRO Solve?
Developers and power users running local services—such as Python APIs, Node.js servers, local Redis caches, Ollama AI models, or background webhooks—frequently suffer from:
- **Terminal Clutter:** Multiple command prompt or PowerShell windows permanently open on the taskbar, vulnerable to accidental closure.
- **Zombie Processes & Resource Leaks:** Closing a terminal often fails to terminate child sub-processes, leaving orphaned processes consuming RAM and locking network ports.
- **Silent Failures:** Background servers that crash or encounter unhandled exceptions go unnoticed until other dependent applications fail.
- **Port Collisions:** Uncertainty about whether a local service is merely running in memory or has actually opened its designated TCP port.
- **Tedious Startup Routines:** Manually navigating multiple project directories and executing startup scripts every time Windows boots.

KAIRO eliminates these headaches by providing a clean **Black + White + Transparent Glass** interface powered by a memory-safe **Rust** process engine. It runs processes hidden from the taskbar, attaches them to Windows Job Objects for guaranteed clean termination, probes port readiness, provides circular in-memory log streaming, and integrates seamlessly with the Windows System Tray and startup registry.

---

## Key Features

- **No Shell Windows:** Spawns binaries directly with Windows `CREATE_NO_WINDOW` flags—zero terminal popups.
- **Guaranteed Tree Kill:** Utilizes native **Windows Job Objects** (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`) to terminate the entire process hierarchy upon stop, preventing orphaned processes.
- **Active TCP Port Readiness Probing:** Distinguishes between a process that is merely alive in RAM (`Running`) and one that is actively accepting connections on its local port (`Ready`).
- **Live System Telemetry:** Displays real-time Process Identifier (PID), CPU usage percentage, private working set memory in MB, and continuous uptime counters.
- **Circular In-Memory Log Streaming:** Captures stdout and stderr streams asynchronously into an isolated 1,000-line circular buffer per service with instant search, stream filtering, and auto-scroll control.
- **Crash Detection & Auto-Restart:** Automatically recovers unexpectedly terminated processes with built-in crash loop protection (rate-limited to 3 attempts within 30 seconds).
- **Windows Logon Integration:** Configures per-user startup via Windows Registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`) without requiring administrator elevation.
- **System Tray Residence:** Minimizes cleanly to the Windows notification tray on window close, keeping services active in the background.
- **Failure Isolation:** Every managed service operates in complete isolation; the failure or crash of one service never impacts any other running service.
- **Automatic Data Migration:** Seamlessly migrates existing service and settings configurations from legacy storage locations without user intervention.

---

## System Requirements

- **Operating System:** Microsoft Windows 10 or Windows 11
- **Supported Architectures:**
  - **Windows 64-bit (x64 / AMD64)**: Native 64-bit execution (Recommended).
  - **Windows 32-bit (x86 / i686)**: Native 32-bit execution for 32-bit Windows systems or WoW64.
- **Dependencies:** Microsoft Visual C++ Redistributable (included in modern Windows updates) and WebView2 Runtime (pre-installed on Windows 10/11).
- **Permissions:** Standard user permissions (Administrator elevation is **not** required).

---

## Download

Choose the installer that matches your Windows system.

### Windows 64-bit

[Download KAIRO for Windows x64](https://github.com/syric28-debug/Kairo/releases/download/v1.0.0/KAIRO-Setup-1.0.0-x64.exe)

For modern 64-bit Windows systems.

### Windows 32-bit

[Download KAIRO for Windows x86](https://github.com/syric28-debug/Kairo/releases/download/v1.0.0/KAIRO-Setup-1.0.0-x86.exe)

For 32-bit Windows systems.

### Which version do I need?

- **x64** = 64-bit Windows
- **x86** = 32-bit Windows

To check:

Windows **Settings → System → About → System type**

[View all KAIRO releases](https://github.com/syric28-debug/Kairo/releases)

---

## Installation

Installing KAIRO is fast, lightweight, and requires no administrative privileges:

1. Download the installer matching your architecture:
   - For 64-bit Windows: `KAIRO-Setup-1.0.0-x64.exe`
   - For 32-bit Windows: `KAIRO-Setup-1.0.0-x86.exe`
2. Double-click the `.exe` installer.
3. The NSIS installer will install KAIRO into your local user directory:  
   `%LOCALAPPDATA%\KAIRO`
4. The installer automatically registers a Start Menu shortcut (`KAIRO`) and includes a clean uninstaller.

> **Screenshot — KAIRO installer**
<!-- Screenshot placeholder: docs/screenshots/01-installer.png -->

---

## First Launch

When KAIRO is launched for the first time, it opens with a clean, hardware-accelerated dark canvas following the **Black + White + Transparent Glass** design language.

If no services have been configured yet, you will see an empty-state dashboard indicating:
- **0 Total Services**
- **0 Running Services**
- **0 Stopped Services**
- **0 Failed Services**
- A **Desktop Engine Active** banner confirming that Windows Job Objects and background monitoring threads are initialized and ready.
- A notification prompting you to add your first service.

> **Screenshot — First Launch Empty State**
<!-- Screenshot placeholder: docs/screenshots/02-first-launch.png -->

---

## Dashboard

The KAIRO Dashboard acts as the primary cockpit for your local environment:

- **Sidebar (Left):** Access high-level navigation tabs:
  - **Dashboard (⊞):** Overview metrics and quick-action table.
  - **Services (▤):** Full service management view with search and filters.
  - **Logs ():** Centralized live terminal output console.
  - **Settings (⚙):** Windows startup and runtime engine preferences.
- **Top Header:** Includes the global search bar, the **`+ Add Service`** action button, and a manual refresh trigger (⟳).
- **Metric Cards (Top Row):**
  - **Total Services:** Total number of configured service profiles.
  - **Running:** Count of currently active child processes (with active Ready count indicator).
  - **Stopped:** Count of idle, unlaunched services.
  - **Failed:** Count of services that crashed or encountered execution errors.
- **Desktop Engine Active Banner:** Verifies that internal process monitors and local loopback IPC pipes are operating normally.
- **Recent Services Table:** Displays quick Start/Stop controls, configured ports, and command pills for recent services.

> **Screenshot — Dashboard**
<!-- Screenshot placeholder: docs/screenshots/03-dashboard.png -->

---

## Adding Your First Service

To register a new local background service in KAIRO, click the **`+ Add Service`** button in the top header. This opens the **Add Generic Service** modal dialog.

> **Screenshot — Add Service Modal**
<!-- Screenshot placeholder: docs/screenshots/04-add-service.png -->

### Configuration Fields Explained

Fill out the configuration parameters according to your service needs:

1. **Service Name (Required):** A recognizable display name (e.g., `Local Redis Server`, `FastAPI Backend`, `Frontend Dev Server`). Maximum 100 characters.
2. **Target Port (Optional):** The TCP network port that this service listens on (between `1` and `65535`). If the service is a background worker without a network listener, leave this field blank.
3. **Description (Optional):** A brief explanation of the service's purpose (e.g., `Main REST API service running on uvicorn`). Maximum 500 characters.
4. **Executable Path or Command (Required):** The exact program binary or system command to run. If the program is registered in your system `PATH` (such as `python`, `node`, or `git`), you can enter the command name directly. Otherwise, provide the full Windows file path (e.g., `C:\Tools\redis-server.exe`).
5. **Command Arguments (Optional):** Command-line arguments and flags passed directly to the executable, separated by spaces (e.g., `-m http.server 8080` or `server.js --port 3000`).
6. **Working Directory (Optional, Defaults to `C:\`):** The folder path where the executable will execute. This is critical for scripts that require access to local project files, configuration files, or local dependencies (e.g., `D:\projects\my-api`). The directory must physically exist on disk.
7. **Environment Variables (Optional):** Custom environment variables injected into the process environment. Enter one `KEY=VALUE` pair per line. Lines beginning with `#` are treated as comments.
8. **Health Check Endpoint / URL (Optional):** An HTTP health check URL (e.g., `http://localhost:8080/health`).
9. **Auto-start on application launch (Checkbox):** When enabled, KAIRO automatically launches this service whenever the KAIRO application starts.
10. **Auto-restart if process terminates unexpectedly (Checkbox):** When enabled, KAIRO detects unexpected crashes and automatically relaunches the process with crash loop rate limiting.

Click **Create Service** to persist the configuration.

> **Screenshot — Configured Service Form**
<!-- Screenshot placeholder: docs/screenshots/04-add-service.png -->

---

## Starting a Service

In the **Services** view or on the **Dashboard**, locate the service card and click the green **`Start` (▶)** button.

### Process State Transitions

When starting, KAIRO transitions through the following lifecycle states:

```
[Stopped] ──> [Starting] ──> [Running] ──> [Ready] (if port listening)
                                │
                                └──> [Failed] (on crash or launch error)
```

1. **Starting:** KAIRO reserves the process handle and asks the operating system to spawn the process with pipes attached.
2. **Running:** The process has been assigned an official Windows **Process ID (PID)** and is executing in RAM. The live PID pill and CPU/Memory metrics bar become active.
3. **Ready:** If a port was specified, KAIRO's background TCP probe successfully establishes a loopback connection. The port tag highlights in green (`:8080 [Ready]`).
4. **Waiting...:** If a port was specified but the service is still initializing (loading models, connecting to databases), the port tag shows a yellow `:port [Waiting...]` status until the socket binds.
5. **Stopped:** The process is dormant with no resources allocated.
6. **Failed:** The executable failed to launch (e.g., path not found) or terminated unexpectedly with an error code.

> **Screenshot — Service Running with Metrics**
<!-- Screenshot placeholder: docs/screenshots/05-service-running.png -->

> **Screenshot — Service Port Ready**
<!-- Screenshot placeholder: docs/screenshots/06-service-ready.png -->

---

## Stopping a Service

To shut down an active service:

1. Click the red **`Stop` (■)** button on the service card.
2. KAIRO signals the service process tree.
3. If the process does not terminate gracefully within the configured **Kill Grace Period** (default: 5,000 ms), the underlying Windows Job Object forcefully terminates the process and all spawned child processes.
4. The PID is released, the metrics bar is removed, and the card status returns to **`stopped`**.

> **Screenshot — Service Stopped**
<!-- Screenshot placeholder: docs/screenshots/07-service-stopped.png -->

---

## Restarting a Service

Clicking the **`Restart` (⟳)** button performs a clean cycle:
1. KAIRO transitions the service to `restarting`.
2. The running process and its child processes are terminated cleanly.
3. Old PID tracking and log streams are finalized.
4. A brand new process is spawned with a newly assigned Windows PID.
5. Telemetry counters reset, and port readiness probing begins anew.

> **Screenshot — Service Restarted**
<!-- Screenshot placeholder: docs/screenshots/08-service-restarted.png -->

---

## Port & Readiness

Unlike basic task managers that simply check whether a process exists in memory, KAIRO verifies **network availability**:

- **Loopback-Only Probing:** Probes are restricted exclusively to `127.0.0.1` and `[::1]` with a non-blocking 150 ms timeout. KAIRO never initiates external network scans.
- **Port Collision Prevention:** When pre-flight port probing is active, KAIRO alerts you if an address is already bound by another process prior to launch.
- **Stale Probe Rejection:** If a service restarts rapidly, probe results targeted at the previous PID are safely discarded, preventing race conditions.

> **Screenshot — Port Readiness Probing**
<!-- Screenshot placeholder: docs/screenshots/09-port-readiness.png -->

---

## Auto Start

KAIRO provides a two-layer startup architecture:

1. **Application Auto Start (`Launch on Windows Boot`):**  
   Located in **Settings**. Registers KAIRO in the Windows per-user Run registry (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`). KAIRO launches silently whenever you log into Windows.
2. **Service Auto Start (`Start configured services on launch`):**  
   Master toggle in **Settings**. When enabled, KAIRO iterates through all configured services and automatically launches any service where `Auto-start on application launch` is checked.

> **Screenshot — Auto Start Settings & Startup Banner**
<!-- Screenshot placeholder: docs/screenshots/10-auto-start.png -->

---

## Auto Restart

When `Auto-restart if process terminates unexpectedly` is enabled on a service:

- **Crash Detection:** KAIRO's background monitor polls process status every second. If a process exits without an explicit user command, KAIRO logs the exit and schedules an immediate restart.
- **Intentional Stop Protection:** Clicking the manual **`Stop`** button signals an intentional shutdown; KAIRO will **never** trigger an auto-restart after an intentional user stop.
- **Crash Loop Protection:** If a faulty binary crashes repeatedly, KAIRO limits auto-restarts to a maximum of **3 consecutive attempts within a 30-second window**. If exceeded, the service is suspended in a `Failed` state to safeguard system CPU and disk resources. Running stably for 10 seconds resets the consecutive attempt counter.

> **Screenshot — Auto Restart Protection**
<!-- Screenshot placeholder: docs/screenshots/11-auto-restart.png -->

---

## System Tray

KAIRO integrates directly with the Windows Taskbar Notification Area (System Tray):

- **Left-Click Tray Icon:** Instantly restores and focuses the KAIRO main dashboard window.
- **Right-Click Tray Menu:**
  - **Open Dashboard:** Restores the main application frame.
  - **Start All:** Sequentially launches all valid stopped services.
  - **Stop All:** Stops all running services cleanly.
  - **Restart All:** Cycles all services with fresh PIDs.
  - **Exit:** Completely terminates KAIRO and all monitoring loops.
- **Close (✕) vs. Exit:**  
  Clicking the window titlebar close button (**✕**) hides the window to the System Tray; your background services continue running uninterrupted. To terminate KAIRO entirely, select **Exit** from the System Tray menu.

> **Screenshot — System Tray Menu**
<!-- Screenshot placeholder: docs/screenshots/12-system-tray.png -->

---

## Logs

Navigate to the **`Logs` ()** tab in the sidebar for real-time terminal output streaming:

- **Service Selector:** Choose `All Services (Aggregated)` or filter output to a specific service profile.
- **Stream Tabs:** Toggle between `All`, `stdout` (normal program output), and `stderr` (warnings and errors).
- **Search:** Instant text search across stored messages and service names.
- **Circular Buffer:** Preserves the most recent 1,000 log entries per service in RAM to prevent memory exhaustion.
- **Controls:**
  - **Clear:** Clears the active in-memory log buffer.
  - **Copy:** Copies formatted log entries to the Windows clipboard.
  - **Smart Scroll:** Auto-scrolls to the newest log line; scrolling up automatically pauses auto-scroll so you can inspect error traces comfortably.

> **Screenshot — Real-Time Log Viewer**
<!-- Screenshot placeholder: docs/screenshots/13-logs.png -->

---

## Settings

The **`Settings` (⚙)** tab provides full control over application preferences and defaults:

1. **General Preferences:**
   - **Visual Design System:** Locked to permanent *Black + White + Transparent Glass*.
   - **Minimize to System Tray:** Controls whether closing the window hides it to the tray.
   - **Launch on Windows Boot:** Manages the Windows `HKCU\Run` registry entry.
   - **Start configured services on launch:** Global master switch for auto-starting eligible services.
   - **Pre-flight Port Probing:** Verifies local ports are free before launching processes.
2. **Process Engine Defaults:**
   - **Kill Grace Period (ms):** Timeout duration (default: `5000` ms) before a non-responsive process tree is forcefully terminated.
3. **Application Information:**
   - Displays compiled version (`1.0.0`), application identifier (`com.kairo.localruntimemanager`), platform target, and technology stack.

> **Screenshot — Settings View**
<!-- Screenshot placeholder: docs/screenshots/14-settings.png -->

---

## Editing a Service

To adjust an existing service's configuration:
1. Click the **Edit** (pencil) icon on the top-right of the service card.
2. The modal dialog opens pre-populated with current values.
3. Update paths, arguments, environment variables, or port configurations.
4. Click **Save Changes**.
5. *Note:* If the service is currently running, changes will take effect upon the next restart.

> **Screenshot — Editing a Service**
<!-- Screenshot placeholder: docs/screenshots/15-edit-service.png -->

---

## Deleting a Service

To permanently delete a service configuration:
1. If the service is currently running or ready, **you must click Stop first**. KAIRO disables the delete action on active services to protect against accidental termination.
2. Once stopped, click the **Delete** (trash can) icon.
3. A confirmation modal will appear.
4. Confirm by clicking **Delete Service**. The configuration is permanently removed from disk storage.

> **Screenshot — Delete Confirmation Modal**
<!-- Screenshot placeholder: docs/screenshots/16-delete-service.png -->

---

## Troubleshooting

### 1. "Executable not found"
- **Cause:** The program name entered is not in your Windows `PATH`, or the file path has a typo.
- **Solution:** Provide the absolute file path (e.g., `C:\Python312\python.exe` or `C:\Program Files\nodejs\node.exe`). Ensure the file exists in that directory.

### 2. Status Stays on "Waiting..." Indefinitely
- **Cause:** The process is running, but it has not opened the configured TCP port.
- **Solution:** Verify the port number configured in KAIRO matches the exact port specified in your application's configuration file or command arguments. Check the **Logs** tab to see if your application failed during initialization.

### 3. Service Starts and Immediately Changes to "Failed"
- **Cause:** The script or binary crashed upon startup due to missing dependencies, syntax errors, or an invalid working directory.
- **Solution:** Open the **Logs** tab, select the failed service, and inspect the red `stderr` lines to see the exact crash stack trace. Verify that the **Working Directory** points to the actual project root.

### 4. Port Collision Warning
- **Cause:** Another program is already listening on the requested TCP port.
- **Solution:** Either stop the conflicting application or assign a different port to your service in the KAIRO edit modal.

### 5. Services Do Not Launch on Windows Boot
- **Cause:** Either the registry startup switch or the master auto-start switch is disabled.
- **Solution:** In KAIRO **Settings**, ensure both `Launch on Windows Boot` and `Start configured services on launch` are switched **ON**. Additionally, verify that `Auto-start on application launch` is checked in the individual service configuration.

---

## Frequently Asked Questions (FAQ)

**Q: Does KAIRO require Administrator privileges?**  
A: No. KAIRO installs in user space (`%LOCALAPPDATA%\KAIRO`), stores settings in `%APPDATA%`, and registers for startup via `HKCU\Run`. It runs entirely with standard user privileges.

**Q: Will closing the KAIRO window kill my running services?**  
A: No. When `Minimize to System Tray` is enabled, closing the window simply hides the interface to the Windows notification tray. Your services continue running. To stop services, use the Stop buttons or choose **Exit** from the tray menu.

**Q: Where are my service configurations stored?**  
A: Service definitions are stored atomically in `%APPDATA%\com.kairo.localruntimemanager\services.json`.

**Q: Can KAIRO run batch scripts (.bat) or PowerShell scripts (.ps1)?**  
A: Yes. To run a batch file, set Executable to `cmd.exe` and Arguments to `/c C:\path\to\script.bat`. To run a PowerShell script, set Executable to `powershell.exe` and Arguments to `-ExecutionPolicy Bypass -File C:\path\to\script.ps1`.

---

## Uninstall

To remove KAIRO completely from your Windows system:

1. Stop all active services in KAIRO and exit the application from the System Tray.
2. Open Windows **Settings → Apps → Installed apps** (or Control Panel → Programs and Features).
3. Locate **KAIRO** and click **Uninstall**.
4. Alternatively, execute the uninstaller directly from:  
   `%LOCALAPPDATA%\KAIRO\uninstall.exe`
5. The uninstaller cleanly removes all application binaries, desktop shortcuts, and Start Menu entries.

> **Screenshot — Windows Uninstall**
<!-- Screenshot placeholder: docs/screenshots/17-uninstall.png -->

---

## Developer / Build From Source

To compile KAIRO from source, ensure your environment meets the prerequisites:

### Technology Stack
- **Desktop Engine:** [Tauri 2](https://v2.tauri.app/)
- **Backend Language:** [Rust](https://www.rust-lang.org/) (2021 Edition, >= 1.75)
- **Frontend Framework:** [React 19](https://react.dev/)
- **Language & Typings:** [TypeScript](https://www.typescriptlang.org/)
- **Bundler & Build Tool:** [Vite](https://vitejs.dev/)
- **Icons:** [Lucide React](https://lucide.dev/)

### Build Commands

```bash
# 1. Install frontend dependencies
npm install

# 2. Typecheck frontend code
npx tsc --noEmit

# 3. Check Rust backend code
cargo check --manifest-path src-tauri/Cargo.toml

# 4. Run automated test suite
cargo test --manifest-path src-tauri/Cargo.toml

# 5. Run in local development mode with hot-reload
npm run tauri dev

# 6. Build production 64-bit installer
npx tauri build --target x86_64-pc-windows-msvc

# 7. Build production 32-bit installer
rustup target add i686-pc-windows-msvc
npx tauri build --target i686-pc-windows-msvc
```

---

## Architecture

KAIRO utilizes a decoupled, high-performance architecture:

```
┌──────────────────────────────────────────────────────────────────┐
│                   React 19 + TypeScript (Vite)                  │
│       Black + White + Transparent Glass Developer Interface       │
└─────────────────────────────────┬────────────────────────────────┘
                                  │
                       Tauri 2 Typed IPC (JSON)
           (invoke commands / typed event stream subscriptions)
                                  │
┌─────────────────────────────────▼────────────────────────────────┐
│                           Rust Backend                           │
│  ┌───────────────────────┐             ┌──────────────────────┐  │
│  │    Commands / IPC     │             │   Services / Core    │  │
│  │ (process, logs, etc.) │             │  (ProcessManager)    │  │
│  └───────────┬───────────┘             └──────────┬───────────┘  │
│              │                                    │              │
│              ▼                                    ▼              │
│  ┌───────────────────────┐             ┌──────────────────────┐  │
│  │      LogManager       │             │   Windows Job Tree   │  │
│  │ (In-memory 1k buffer) │             │ (Clean Kill-Tree)    │  │
│  └───────────────────────┘             └──────────────────────┘  │
│              ▲                                    │              │
│              │                                    ▼              │
│  ┌───────────────────────┐             ┌──────────────────────┐  │
│  │   Stream Readers      │             │     PortChecker      │  │
│  │  (Async stdout/err)   │             │   (Localhost TCP)    │  │
│  └───────────────────────┘             └──────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

- **Process Isolation:** Processes are spawned directly via `std::process::Command` without shell wrappers.
- **Windows Job Objects:** Every process hierarchy is contained in an anonymous Windows Job Object to guarantee full tree-kill upon stopping.
- **Port Checker:** Bounded loopback probing checks network readiness without external network access.
- **Atomic Persistence:** Configuration data is stored using atomic write-and-rename mechanics with automatic migration from legacy paths.

---

## License

This project is currently unlicensed / proprietary to the author. See the repository terms for permissions and usage guidelines.
