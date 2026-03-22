---
phase: 05-flatpak-distirbution-and-automatic-update
plan: 03
subsystem: infra
tags: [github-actions, flatpak, ci, ostree, gh-pages, flatpakref, release-pipeline, git-lfs]

requires:
  - phase: 05-01
    provides: Flatpak manifest at flatpak/io.github.maartenp.puuzel.yml
  - phase: 05-02
    provides: In-app version check module (src/update.rs)
provides:
  - GitHub Actions CI pipeline (tag push -> Flatpak build -> GitHub Pages deploy)
  - .flatpakref first-install file for dad pointing at OSTree remote on GitHub Pages
  - version.txt at raw.githubusercontent.com for in-app update check
affects: [DIST-01, DIST-02]

tech-stack:
  added: [andyholmes/flatter, actions/deploy-pages, git-lfs]
  patterns:
    - "Flatter with upload-pages-artifact: true handles Pages artifact directly"
    - "cargo-sources.json pre-generated locally, committed to repo"
    - "freedesktop:24.08 container image matches SDK runtime-version 24.08"
    - "version.txt served from raw.githubusercontent.com/main"

key-files:
  created:
    - .github/workflows/release.yml
    - io.github.maartenp.puuzel.flatpakref
    - flatpak/io.github.maartenp.puuzel.svg
    - flatpak/cargo-sources.json
    - version.txt
    - .gitattributes
  modified:
    - flatpak/io.github.maartenp.puuzel.yml
    - src/update.rs
    - release.sh

key-decisions:
  - "Pre-generate cargo-sources.json locally — flatter container has no pip/pip3"
  - "version.txt served from raw.githubusercontent.com instead of GitHub Pages"
  - "puuzel.db tracked via Git LFS (89MB exceeds GitHub file limit)"
  - ".flatpakref has no GPGKey field — unsigned repo; GNOME Software shows untrusted source confirmation (expected)"

patterns-established:
  - "Release flow: release.sh bumps version + version.txt -> tag -> push -> CI builds Flatpak -> Pages deploy"
  - "Regenerate cargo-sources.json when Cargo.lock changes"

requirements-completed: [DIST-01, DIST-02]

duration: 45min
completed: 2026-03-22
---

# Phase 05 Plan 03: CI Release Pipeline and .flatpakref Summary

**Tag-triggered CI pipeline builds Flatpak via andyholmes/flatter, deploys OSTree repo to GitHub Pages, with .flatpakref for dad's first install**

## Performance

- **Duration:** ~45 min (including iterative CI debugging)
- **Tasks:** 2/2 complete
- **Files created:** 6
- **Files modified:** 3

## Accomplishments

- GitHub Actions workflow triggers on `v*` tag push, builds Flatpak, deploys to Pages
- .flatpakref file enables one-click Flatpak install on Linux Mint
- version.txt accessible at raw.githubusercontent.com for in-app update checks
- App icon SVG added for AppStream metadata validation
- cargo-sources.json pre-generated for offline Flatpak build
- Git LFS configured for puuzel.db (89MB)

## Task Commits

1. **Task 1: Create GitHub Actions workflow and .flatpakref** - `7b6b26a` (feat)
2. **Task 2: Verify pipeline (human checkpoint)** - approved after CI runs successfully

**Iterative CI fixes:**
- `b1e7dfe` fix: pip → pip3 (still not available in container)
- `ca8d97a` fix: pre-generate cargo-sources.json, remove CI generation
- `e38da06` fix: cargo-sources.json path + rust SDK extension install
- `214c418` fix: commit uncommitted source files from earlier phases
- `682d339` feat: add data files via Git LFS
- `b7f9c17` fix: enable LFS checkout in CI
- `1fa808b` fix: add app icon SVG for AppStream validation
- `4c726a3` fix: simplify version.txt to raw.githubusercontent.com

## Files Created/Modified

- `.github/workflows/release.yml` - CI pipeline: tag push -> Flatpak build -> GitHub Pages deploy
- `io.github.maartenp.puuzel.flatpakref` - First-install file for dad
- `flatpak/io.github.maartenp.puuzel.svg` - App icon for AppStream
- `flatpak/cargo-sources.json` - Pre-generated crate sources for offline build
- `version.txt` - Current version for in-app update check
- `.gitattributes` - Git LFS tracking rules
- `flatpak/io.github.maartenp.puuzel.yml` - Fixed source path reference
- `src/update.rs` - Changed URL to raw.githubusercontent.com
- `release.sh` - Now writes version.txt during release

## Decisions Made

- **cargo-sources.json pre-generated:** Flatter container has no pip/pip3. Generated locally via flatpak-cargo-generator.py. Regenerate when Cargo.lock changes.
- **version.txt via raw.githubusercontent.com:** Injecting files into the OSTree Pages artifact was unreliable. Raw GitHub URLs are simpler and always available.
- **Git LFS for puuzel.db:** 89MB exceeds GitHub's 100MB file limit. Font (739K) committed directly.
- **Unsigned .flatpakref:** No GPGKey field. GNOME Software shows untrusted source confirmation — expected and acceptable.

## Deviations from Plan

- Plan expected cargo-sources.json generation in CI — moved to pre-generation
- Plan expected version.txt on GitHub Pages — moved to raw.githubusercontent.com
- Multiple CI fixes required due to flatter container constraints

## Issues Encountered

- Flatter container lacks pip/pip3 — resolved by pre-generating cargo-sources.json
- Source path doubled (flatpak/flatpak/) — fixed relative path reference
- Rust SDK extension not pre-installed — added explicit install step
- Uncommitted source files caused build failures — committed missing changes
- Missing app icon failed AppStream validation — created SVG icon
- Pages artifact upload failed with empty path — let flatter handle it directly
- GitHub Pages environment protection rules blocked tag deploys — user added v* rule

## Next Phase Readiness

- Flatpak distribution pipeline fully operational
- Release flow: `./release.sh` → tag push → CI build → GitHub Pages deploy
- Dad can install via .flatpakref file

---
*Phase: 05-flatpak-distirbution-and-automatic-update*
*Completed: 2026-03-22*

## Self-Check: PASSED

- FOUND: .github/workflows/release.yml
- FOUND: io.github.maartenp.puuzel.flatpakref
- FOUND: flatpak/io.github.maartenp.puuzel.svg
- FOUND: flatpak/cargo-sources.json
- FOUND: version.txt
