---
phase: 05-flatpak-distirbution-and-automatic-update
plan: 03
subsystem: infra
tags: [github-actions, flatpak, ci, ostree, gh-pages, flatpakref, release-pipeline]

# Dependency graph
requires:
  - phase: 05-01
    provides: Flatpak manifest at flatpak/io.github.maartenp.puuzel.yml
  - phase: 05-02
    provides: version check URL https://maartenp.github.io/puuzel/version.txt
provides:
  - GitHub Actions CI pipeline (tag push -> cargo-sources.json -> Flatpak build -> version.txt inject -> GitHub Pages deploy)
  - .flatpakref first-install file for dad pointing at OSTree remote on GitHub Pages
affects: [DIST-01, DIST-02, phase-04-macos]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Flatter with upload-pages-artifact: false + manual version.txt inject into OSTree repo dir"
    - "flatpak-cargo-generator.py cloned from flatpak-builder-tools, run before flatter build step"
    - "freedesktop:24.08 container image matches SDK runtime-version 24.08"

key-files:
  created:
    - .github/workflows/release.yml
    - io.github.maartenp.puuzel.flatpakref
  modified: []

key-decisions:
  - "upload-pages-artifact: false on Flatter — manual inject of version.txt into steps.flatter.outputs.repo before upload-pages-artifact@v3"
  - ".flatpakref has no GPGKey field — unsigned repo per D-02; GNOME Software shows untrusted source confirmation (expected)"
  - "pip install aiohttp tomlkit for flatpak-cargo-generator.py prerequisites (tomlkit is the correct package name)"

patterns-established:
  - "CI Pattern: generate cargo-sources.json in CI (D-13), build Flatpak, inject version.txt into OSTree repo directory, deploy to GitHub Pages"

requirements-completed: [DIST-01, DIST-02]

# Metrics
duration: 1min
completed: 2026-03-22
---

# Phase 05 Plan 03: CI Release Pipeline and .flatpakref Summary

**GitHub Actions workflow that generates cargo-sources.json, builds Flatpak via andyholmes/flatter, injects version.txt into OSTree repo dir before Pages deploy, plus .flatpakref for dad's one-click install**

## Performance

- **Duration:** ~1 min
- **Started:** 2026-03-22T17:26:05Z
- **Completed:** 2026-03-22T17:27:08Z (Task 1 of 2; Task 2 is human checkpoint)
- **Tasks:** 1 of 2 automated (Task 2 is checkpoint:human-verify)
- **Files modified:** 2

## Accomplishments

- Created `.github/workflows/release.yml` triggering on `v*` tag push: generates `cargo-sources.json` from `Cargo.lock`, builds Flatpak via `andyholmes/flatter@main`, writes `version.txt` into `${{ steps.flatter.outputs.repo }}` before uploading pages artifact, deploys to GitHub Pages
- Created `io.github.maartenp.puuzel.flatpakref` pointing at `https://maartenp.github.io/puuzel/` with `RuntimeRepo=https://flathub.org/repo/flathub.flatpakrepo` and no GPGKey field (unsigned per D-02)
- Confirmed `cargo build --release` still succeeds (no regressions from these new files)

## Task Commits

1. **Task 1: Create GitHub Actions release workflow and .flatpakref** - `7b6b26a` (feat)

**Task 2 (checkpoint:human-verify):** Awaiting user to verify GitHub repo setup, enable Pages, review files, push first release, and confirm version.txt is reachable at https://maartenp.github.io/puuzel/version.txt

## Files Created/Modified

- `.github/workflows/release.yml` - CI pipeline: tag push -> cargo-sources.json generation -> Flatpak build -> version.txt inject -> GitHub Pages publish
- `io.github.maartenp.puuzel.flatpakref` - First-install file for dad (double-click opens GNOME Software)

## Decisions Made

- Used `upload-pages-artifact: false` on Flatter + `actions/upload-pages-artifact@v3` manually — this is the only reliable way to inject `version.txt` into the pages artifact so DIST-02 (version check) works after deploy
- `.flatpakref` contains no `GPGKey` field — unsigned repo per D-02; GNOME Software will show "untrusted source" confirmation which is expected
- `tomlkit` used as Python dependency (not `toml`) — this is the package name required by flatpak-cargo-generator.py

## Deviations from Plan

None — plan executed exactly as written for Task 1.

## Issues Encountered

None.

## User Setup Required (Task 2 checkpoint)

1. Verify GitHub repo exists and remote is configured (`git remote -v`)
2. Enable GitHub Pages: Settings -> Pages -> Source: "GitHub Actions"
3. Review `.github/workflows/release.yml`, `flatpak/io.github.maartenp.puuzel.yml`, `io.github.maartenp.puuzel.flatpakref`, and `release.sh`
4. Run `./release.sh` to tag and push v0.1.0 (or `git tag v0.1.0 && git push && git push --tags`)
5. Watch GitHub Actions to confirm workflow succeeds
6. After deploy: `curl https://maartenp.github.io/puuzel/version.txt` should return `0.1.0`

## Next Phase Readiness

- All three Phase 05 plans deliver DIST-01, DIST-02, DIST-04 together
- After first tag push and successful CI run, the Flatpak distribution pipeline is fully operational
- Dad can double-click `io.github.maartenp.puuzel.flatpakref` on Linux Mint to install via GNOME Software

---
*Phase: 05-flatpak-distirbution-and-automatic-update*
*Completed: 2026-03-22 (partial — checkpoint at Task 2)*

## Self-Check: PASSED

- FOUND: .github/workflows/release.yml
- FOUND: io.github.maartenp.puuzel.flatpakref
- FOUND commit 7b6b26a (Task 1)
