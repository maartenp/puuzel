---
phase: 05-flatpak-distirbution-and-automatic-update
plan: 02
subsystem: infra
tags: [ureq, flatpak, version-check, update-notification, data-paths]

# Dependency graph
requires:
  - phase: 05-01
    provides: Flatpak manifest with /app/share/puuzel/ install layout
provides:
  - In-app startup version check via ureq background thread
  - Dutch update notification overlay (draw_update_notification)
  - Flatpak-aware data paths for DB and font (Pitfall 7 fix)
affects: [05-03, flatpak-build, ci-release]

# Tech tracking
tech-stack:
  added: [ureq = "3"]
  patterns:
    - "Flatpak path detection: check /app/share/puuzel/ first, fall back to data/ for dev"
    - "Version check: background thread + mpsc::Receiver<Option<String>>, silently returns None on any network failure"

key-files:
  created:
    - src/update.rs
  modified:
    - Cargo.toml
    - src/main.rs
    - src/render/overlay.rs
    - src/render/mod.rs

key-decisions:
  - "ureq 3.x read_to_string() takes 0 arguments and returns Result<String> — plan code used 0.2.x/2.x API with &mut body arg; fixed inline"
  - "update_dismissed flag prevents re-showing the notification after the user dismisses it in the same session"
  - "Version comparison uses simple != (string equality) per research recommendation — semver parsing not needed for this use case"

patterns-established:
  - "Flatpak path detection: check /app/share/puuzel/<file>.exists() at runtime, fall back to data/<file> for dev"
  - "Silent version check: spawn_version_check() wraps all network ops in Option closure — any error returns None, never panics or logs error"

requirements-completed: [DIST-02]

# Metrics
duration: 3min
completed: 2026-03-22
---

# Phase 05 Plan 02: Version Check and Flatpak Data Paths Summary

**ureq 3 background version check with Dutch update overlay and /app/share/puuzel/ Flatpak path detection for DB and font**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-03-22T16:21:35Z
- **Completed:** 2026-03-22T16:24:02Z
- **Tasks:** 2 of 2
- **Files modified:** 5

## Accomplishments

- Added `ureq = "3"` and created `src/update.rs` with `spawn_version_check()` that fetches version.txt from GitHub Pages in a background thread, returning `None` on any network/parse failure
- Integrated version check into main loop — polls non-blocking via `try_recv`, compares to `env!("CARGO_PKG_VERSION")`, sets `update_available` state
- Added `draw_update_notification()` to `src/render/overlay.rs` — Dutch dialog ("Nieuwe versie beschikbaar" / "Voer 'flatpak update' uit om bij te werken.") with OK dismiss button
- Fixed Flatpak data path resolution in both `src/main.rs` (DB) and `src/render/mod.rs` (font) — checks `/app/share/puuzel/` first, falls back to `data/` for dev

## Task Commits

1. **Task 1: Add ureq dependency and create version check module** - `27a2c45` (feat)
2. **Task 2: Integrate version check into main loop, add update overlay, fix Flatpak data paths** - `fc64d29` (feat)

## Files Created/Modified

- `src/update.rs` - New module: `spawn_version_check()` returns `mpsc::Receiver<Option<String>>`; uses ureq 3.x blocking GET in background thread
- `Cargo.toml` - Added `ureq = "3"` dependency
- `src/main.rs` - Flatpak-aware `db_path`, version check spawn + poll loop, update overlay render call
- `src/render/overlay.rs` - New `draw_update_notification()` function with Dutch message and OK button
- `src/render/mod.rs` - Flatpak-aware font path in `init_font()`

## Decisions Made

- Used ureq 3.x API: `read_to_string()` takes no arguments and returns `Result<String, Error>` — the plan's code example used an older API signature with `&mut body` arg; auto-fixed inline.
- `update_dismissed` flag added alongside `update_available` to prevent re-showing the notification after user clicks OK in the same session.
- Version comparison via simple string `!=` (no semver parsing) per research recommendation (D-11 equivalent).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ureq 3.x read_to_string API mismatch**
- **Found during:** Task 1 (version check module creation)
- **Issue:** Plan's code example called `.read_to_string(&mut body)` using the old std::io::Read trait pattern, but ureq 3.x exposes its own `read_to_string()` method that takes 0 arguments and returns `Result<String>` directly
- **Fix:** Removed `use std::io::Read;` import, changed call to `let body = ...read_to_string().ok()?`
- **Files modified:** `src/update.rs`
- **Verification:** `cargo check` passed after fix
- **Committed in:** `27a2c45` (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - API incompatibility)
**Impact on plan:** Required fix for correctness — no scope creep.

## Issues Encountered

None beyond the ureq API mismatch documented above.

## User Setup Required

None - no external service configuration required for this plan. The `version.txt` file on GitHub Pages is created by Plan 03 (CI workflow).

## Next Phase Readiness

- Version check code is complete and compiles; it will silently return `None` until Plan 03's CI creates `version.txt` on GitHub Pages
- Flatpak data paths are correct — DB and font will resolve to `/app/share/puuzel/` when running inside the sandbox
- Plan 03 (CI + release workflow) can proceed without further changes to the app code

---
*Phase: 05-flatpak-distirbution-and-automatic-update*
*Completed: 2026-03-22*
