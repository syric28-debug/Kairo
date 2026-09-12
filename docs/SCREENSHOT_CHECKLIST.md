# KAIRO — Screenshot Checklist (v1.0.0)

This checklist covers every screenshot required for the KAIRO README.

**Rules (do not violate):**
- Capture screenshots **only from the real, running KAIRO application** on Windows.
- Do **not** generate, mock, or edit screenshots from other sources.
- Save captures as `.png` files with the exact filenames below into `docs/screenshots/`.
- When a file is captured and added, replace the matching
  `<!-- Screenshot placeholder: ... -->` comment in `README.md` with a real
  `![...](docs/screenshots/<file>)` image embed.
- Recommended capture settings: default window size (960×640 minimum), default
  **Black + White + Transparent Glass** theme, 100% Windows display scaling.

| # | Filename | Exact KAIRO screen to open | What must be visible | What should be highlighted | Where it belongs in README |
|---|----------|----------------------------|----------------------|----------------------------|----------------------------|
| 1 | `01-installer.png` | KAIRO NSIS installer wizard, first page | Installer window, product name "KAIRO", version `1.0.0`, license/consent screen | The install button and per-user install notice | [Installation](#installation) |
| 2 | `02-first-launch.png` | KAIRO launched with no services configured | Empty-state dashboard: 0 Total / 0 Running / 0 Stopped / 0 Failed metric cards, "Desktop Engine Active" banner, add-first-service prompt | The empty-state metric cards and banner | [First Launch](#first-launch) |
| 3 | `03-dashboard.png` | Dashboard tab with at least 2–3 configured services | Sidebar (Dashboard / Services / Logs / Settings), top search bar, `+ Add Service` button, metric cards, Recent Services table | One running service row and the metric cards | [Dashboard](#dashboard) |
| 4 | `04-add-service.png` | `+ Add Service` modal with a fully filled example | All fields: Service Name, Target Port, Description, Executable, Arguments, Working Directory, Environment Variables, Health Check URL, both auto-start/auto-restart checkboxes | Required fields and the Create Service button | [Adding Your First Service](#adding-your-first-service) |
| 5 | `05-service-running.png` | Services view (or Dashboard) right after clicking Start on a service | Status pill `running`, live PID pill, CPU/memory metrics bar, Start/Stop/Restart controls | The green status pill and PID/metrics bar | [Starting a Service](#starting-a-service) |
| 6 | `06-service-ready.png` | Same service after its port opens | Port tag showing `:8080 [Ready]` (green), status `ready` | The green port-ready tag | [Starting a Service](#starting-a-service) / [Port & Readiness](#port--readiness) |
| 7 | `07-service-stopped.png` | Services view right after clicking Stop | Status pill `stopped`, PID pill and metrics bar removed | The stopped status pill | [Stopping a Service](#stopping-a-service) |
| 8 | `08-service-restarted.png` | Services view right after clicking Restart | New PID pill (different from PID captured in screenshot 5), metrics reset, status `running`/`ready` | The changed PID value | [Restarting a Service](#restarting-a-service) |
| 9 | `09-port-readiness.png` | Service with a port configured, in transitional state | Port tag showing `:8080 [Waiting...]` (yellow), then optionally side-by-side with the Ready state | The Waiting → Ready transition | [Port & Readiness](#port--readiness) |
| 10 | `10-auto-start.png` | Settings tab | `Launch on Windows Boot` toggle, `Start configured services on launch` master toggle, and a service's `Auto-start on application launch` checkbox | Both startup toggles in the ON state | [Auto Start](#auto-start) |
| 11 | `11-auto-restart.png` | Logs view after a service with Auto-restart crashed and was relaunched | The auto-restart log entries (crash detected + restart), status `running` again | The crash/restart log lines | [Auto Restart](#auto-restart) |
| 12 | `12-system-tray.png` | Windows taskbar notification area with KAIRO tray icon | Tray icon with the right-click menu open: Open Dashboard, Start All, Stop All, Restart All, Exit | The expanded tray context menu | [System Tray](#system-tray) |
| 13 | `13-logs.png` | Logs tab with a running service producing output | Service selector, All/stdout/stderr stream tabs, search field, live log lines, Clear/Copy controls, auto-scroll | The stderr stream tab with error lines visible | [Logs](#logs) |
| 14 | `14-settings.png` | Settings tab | General preferences (Minimize to System Tray, Launch on Windows Boot, Start configured services on launch, Pre-flight Port Probing), Kill Grace Period, Application Information (version `1.0.0`, identifier `com.kairo.localruntimemanager`) | The Application Information block | [Settings](#settings) |
| 15 | `15-edit-service.png` | Edit modal opened from a service card's pencil icon | Modal pre-populated with existing values | The Save Changes button and pre-filled fields | [Editing a Service](#editing-a-service) |
| 16 | `16-delete-service.png` | Delete confirmation modal opened from a stopped service's trash icon | Confirmation dialog with the Delete Service button | The confirmation modal | [Deleting a Service](#deleting-a-service) |
| 17 | `17-uninstall.png` | Windows Settings → Apps → Installed apps (or Control Panel → Programs and Features) | "KAIRO" entry with the Uninstall button visible | The KAIRO entry in the installed-apps list | [Uninstall](#uninstall) |

---

## Capture order suggestion

1. Install KAIRO using the x64 installer → capture **01**.
2. First launch → capture **02**.
3. Add 2–3 example services (e.g., `python -m http.server 8080`, a small Node script) → capture **04**.
4. Capture **03** (Dashboard with services configured).
5. Start a service with a port → capture **05**, then **09** (Waiting), then **06** (Ready).
6. Note the PID → Stop → capture **07** → Start again → Restart → capture **08**.
7. Enable auto-restart on one service, force a crash (e.g., kill the child process) → capture **11** from the Logs view.
8. Open Settings → capture **10** and **14**.
9. Open the Logs tab with active output → capture **13**.
10. Edit and delete flows → capture **15** and **16**.
11. Minimize to tray, right-click the tray icon → capture **12**.
12. Open Windows Apps settings → capture **17**.

