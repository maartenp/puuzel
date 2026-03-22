# Phase 5: Flatpak Fast-Path Delivery - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Package and distribute the app as a Flatpak so dad can install it now, and receive automatic updates as development continues. Covers: GitHub repo setup, CI build pipeline, OSTree remote on GitHub Pages, in-app update notification at startup, release script, and the `.flatpakref` install file. macOS build and Flathub submission are deferred to Phase 4.

</domain>

<decisions>
## Implementation Decisions

### Auto-update hosting
- **D-01:** GitHub repo created for this project; GitHub Pages hosts the OSTree remote (zero cost, public access is fine)
- **D-02:** Updates are unsigned — GPG signing deferred to Phase 4 / Flathub submission
- **D-03:** Dad runs `flatpak update` to apply updates; the app prompts him to do so when a newer version is detected

### In-app update notification
- **D-04:** App checks for updates at startup only (not mid-puzzle) by fetching a static `version.txt` from GitHub Pages
- **D-05:** If a newer version is detected, app shows a dialog/notification: "Er is een nieuwe versie beschikbaar. Voer 'flatpak update' uit om bij te werken." (Dutch, simple)
- **D-06:** Version check is a lightweight HTTP GET to a static file — not a Claude API call, not gameplay-blocking
- **D-07:** CLAUDE.md constraint "no custom update logic" is updated: startup version checks via static file fetch are permitted; full in-app updaters are still not allowed

### First-install experience
- **D-08:** Dad installs via a `.flatpakref` file — double-click opens GNOME Software → install dialog. No terminal required for install
- **D-09:** Single machine, no multi-machine install concerns
- **D-10:** `.flatpakref` file points at the GitHub Pages OSTree remote so future updates flow automatically

### Build and release workflow
- **D-11:** `release.sh` script at repo root: reads current version from `Cargo.toml`, shows it, prompts user to choose major/minor/patch increment, updates `Cargo.toml`, commits, creates git tag, and pushes
- **D-12:** GitHub Actions CI triggers on version tag push (`v*`), runs `flatpak-cargo-generator.py` against `Cargo.lock` to produce `cargo-sources.json`, builds the Flatpak, publishes updated OSTree repo to GitHub Pages
- **D-13:** `cargo-sources.json` is NOT committed to the repo — CI generates it fresh on each release from the current `Cargo.lock`

### Phase scope boundary
- **D-14:** Phase 5 runs immediately after Phase 2 (before Phase 3: Session Continuity)
- **D-15:** Phase 4 (Distribution) becomes: macOS build (DIST-03), Flathub submission, and any remaining DIST requirements not covered here
- **D-16:** DIST-01 (Flatpak packages), DIST-02 (auto-updates), DIST-04 (crates.io-only deps) are delivered by Phase 5; DIST-03 (macOS) remains in Phase 4

### Claude's Discretion
- Flatpak application ID (e.g., `nl.maarten.Puuzel` or `io.github.{username}.puuzel`)
- AppStream metadata content (`metainfo.xml`) — minimal but valid for Flatpak build
- Exact OSTree repo layout and GitHub Pages branch (`gh-pages`)
- GitHub Actions runner image and caching strategy
- Version check timeout / failure handling (if offline, silently skip)

</decisions>

<specifics>
## Specific Ideas

- Release script feel: interactive, shows "Current version: 0.3.1 — increment [M]ajor / [m]inor / [p]atch?" and confirms before tagging
- Update notification should be in Dutch, calm in tone — not alarming. Something like a soft banner or dialog, not a modal that blocks play
- Dad is on Linux Mint which ships with Flatpak + GNOME Software pre-installed — no special setup needed on his machine before receiving the `.flatpakref`

</specifics>

<canonical_refs>
## Canonical References

### Distribution requirements
- `.planning/REQUIREMENTS.md` §Distribution — DIST-01 (Flatpak packaging), DIST-02 (auto-updates via Flatpak tooling), DIST-04 (crates.io-only deps for offline build)

### Tech stack constraints
- `CLAUDE.md` §Technology Stack — `flatpak-cargo-generator.py` usage, `org.freedesktop.Sdk` runtime + rust-stable extension, `CARGO_NET_OFFLINE=true` in build options, `rusqlite bundled` feature
- `CLAUDE.md` §What NOT to Use — "Custom update logic inside the app" row is being updated by this phase (startup version check via static file is now permitted)
- `CLAUDE.md` §Flatpak Distribution — Runtime, auto-updates, AppStream metadata sections

### Prior phase decisions
- `.planning/phases/02-playable-game/02-CONTEXT.md` — current app structure and game states (this phase adds a startup version-check state)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/main.rs` game loop entry point — version check runs here at startup, before the first frame
- `src/render/overlay.rs` — existing overlay pattern (congratulations screen); update notification can follow the same pattern

### Established Patterns
- `GameState` enum — may need a `CheckingForUpdate` variant, or the check runs synchronously before entering the game loop
- Background thread + `mpsc::channel` pattern already used for grid generation — version check HTTP call should use the same pattern to avoid blocking the UI thread

### Integration Points
- `Cargo.toml` — version field is the single source of truth; `release.sh` reads and writes it
- New files needed: `flatpak/com.*.puuzel.yml` (manifest), `flatpak/com.*.puuzel.metainfo.xml`, `.github/workflows/release.yml`, `release.sh`

</code_context>

<deferred>
## Deferred Ideas

- GPG signing of the Flatpak remote — Phase 4 / Flathub submission
- macOS build (DMG or Homebrew) — Phase 4
- Flathub submission — Phase 4
- Automatic update download/apply without user running `flatpak update` — explicitly excluded (conflicts with Flatpak trust model)

</deferred>

---

*Phase: 05-flatpak-distirbution-and-automatic-update*
*Context gathered: 2026-03-22*
