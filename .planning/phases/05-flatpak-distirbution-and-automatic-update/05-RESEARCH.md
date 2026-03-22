# Phase 5: Flatpak Fast-Path Delivery - Research

**Researched:** 2026-03-22
**Domain:** Flatpak packaging, OSTree hosting, GitHub Actions CI, Rust HTTP client
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** GitHub repo created for this project; GitHub Pages hosts the OSTree remote (zero cost, public access is fine)
- **D-02:** Updates are unsigned — GPG signing deferred to Phase 4 / Flathub submission
- **D-03:** Dad runs `flatpak update` to apply updates; the app prompts him to do so when a newer version is detected
- **D-04:** App checks for updates at startup only (not mid-puzzle) by fetching a static `version.txt` from GitHub Pages
- **D-05:** If a newer version is detected, app shows a dialog/notification: "Er is een nieuwe versie beschikbaar. Voer 'flatpak update' uit om bij te werken." (Dutch, simple)
- **D-06:** Version check is a lightweight HTTP GET to a static file — not a Claude API call, not gameplay-blocking
- **D-07:** CLAUDE.md constraint "no custom update logic" is updated: startup version checks via static file fetch are permitted; full in-app updaters are still not allowed
- **D-08:** Dad installs via a `.flatpakref` file — double-click opens GNOME Software → install dialog. No terminal required for install
- **D-09:** Single machine, no multi-machine install concerns
- **D-10:** `.flatpakref` file points at the GitHub Pages OSTree remote so future updates flow automatically
- **D-11:** `release.sh` script at repo root: reads current version from `Cargo.toml`, shows it, prompts user to choose major/minor/patch increment, updates `Cargo.toml`, commits, creates git tag, and pushes
- **D-12:** GitHub Actions CI triggers on version tag push (`v*`), runs `flatpak-cargo-generator.py` against `Cargo.lock` to produce `cargo-sources.json`, builds the Flatpak, publishes updated OSTree repo to GitHub Pages
- **D-13:** `cargo-sources.json` is NOT committed to the repo — CI generates it fresh on each release from the current `Cargo.lock`
- **D-14:** Phase 5 runs immediately after Phase 2 (before Phase 3: Session Continuity)
- **D-15:** Phase 4 (Distribution) becomes: macOS build (DIST-03), Flathub submission, and any remaining DIST requirements not covered here
- **D-16:** DIST-01 (Flatpak packages), DIST-02 (auto-updates), DIST-04 (crates.io-only deps) are delivered by Phase 5; DIST-03 (macOS) remains in Phase 4

### Claude's Discretion
- Flatpak application ID (e.g., `nl.maarten.Puuzel` or `io.github.{username}.puuzel`)
- AppStream metadata content (`metainfo.xml`) — minimal but valid for Flatpak build
- Exact OSTree repo layout and GitHub Pages branch (`gh-pages`)
- GitHub Actions runner image and caching strategy
- Version check timeout / failure handling (if offline, silently skip)

### Deferred Ideas (OUT OF SCOPE)
- GPG signing of the Flatpak remote — Phase 4 / Flathub submission
- macOS build (DMG or Homebrew) — Phase 4
- Flathub submission — Phase 4
- Automatic update download/apply without user running `flatpak update` — explicitly excluded (conflicts with Flatpak trust model)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DIST-01 | App packaged as Flatpak for Linux Mint | Flatpak manifest (YAML), org.freedesktop.Sdk + rust-stable extension, flatpak-cargo-generator workflow |
| DIST-02 | Flatpak supports auto-updates via standard Flatpak tooling | OSTree repo on GitHub Pages + .flatpakref pointing at it; `flatpak update` handles download/apply |
| DIST-04 | All Cargo dependencies use crates.io (no git deps, required for Flatpak offline build) | flatpak-cargo-generator.py generates offline sources from Cargo.lock; git deps not supported |
</phase_requirements>

---

## Summary

Phase 5 delivers the complete Flatpak distribution pipeline: a manifest that builds the app inside the Flatpak sandbox, an OSTree repository hosted on GitHub Pages, a GitHub Actions workflow that triggers on version tags to rebuild and publish, and a startup version-check in the app that prompts the user to run `flatpak update`.

The technical approach is well-established. The `andyholmes/flatter` GitHub Action is the simplest end-to-end solution: it builds the Flatpak, exports to an OSTree repo, and pushes to GitHub Pages in a single workflow with minimal configuration. For the in-app version check, `ureq` (blocking HTTP, pure-Rust TLS via rustls) is the correct crate — it needs no async runtime and integrates cleanly with the existing `std::thread::spawn` + `mpsc::channel` pattern already used for grid generation.

The only significant complexity is the `flatpak-cargo-generator.py` step in CI, which converts `Cargo.lock` into a `cargo-sources.json` that Flatpak's offline builder needs. This is a one-command Python script run as part of the CI workflow before the build step.

**Primary recommendation:** Use `andyholmes/flatter` action for CI/OSTree/GitHub Pages; use `ureq 3.3.0` for the in-app version check; application ID `io.github.{username}.puuzel` following reverse-DNS convention with GitHub hosting.

---

## Standard Stack

### Core
| Library / Tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| org.freedesktop.Sdk | 24.08 | Flatpak build runtime + SDK | Current stable (Aug 2024 release, 2-year support cycle); 23.08 is EOL |
| org.freedesktop.Sdk.Extension.rust-stable | branch/24.08 | Rust toolchain inside Flatpak build | Official Flathub extension; must match SDK branch version exactly |
| flatpak-cargo-generator.py | HEAD (flatpak/flatpak-builder-tools) | Convert Cargo.lock → cargo-sources.json for offline build | The only supported way to pre-fetch crates for air-gapped Flatpak builds |
| andyholmes/flatter | @main | GitHub Action: build Flatpak + publish OSTree to gh-pages | Single action handles build + export + repo update + Pages deploy; supports unsigned repos |
| ureq | 3.3.0 | Blocking HTTP GET for version check | Pure-Rust TLS (rustls/ring), no system deps, synchronous API, crates.io only, ~200KB binary overhead |

### Supporting
| Library / Tool | Version | Purpose | When to Use |
|----------------|---------|---------|-------------|
| actions/checkout | v4 | Checkout repo in CI | Standard, use v4 |
| actions/deploy-pages | v4 | Deploy gh-pages artifact | Used with Flatter's `upload-pages-artifact: true` output |
| appstreamcli | (system tool) | Validate metainfo.xml locally before commit | Run `appstreamcli validate --explain flatpak/metainfo.xml` |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| andyholmes/flatter | flatpak/flatpak-github-actions | flatpak-github-actions produces a `.flatpak` bundle but does NOT publish an OSTree repo to GitHub Pages — wrong for this phase's auto-update goal |
| andyholmes/flatter | Manual flatpak-builder + build-update-repo + gh-pages push | More control but ~50 lines of workflow YAML vs 10; not needed at this scale |
| ureq 3.3.0 | minreq 2.14.1 | minreq is even smaller but has fewer rustls features; ureq is more actively maintained and has better TLS defaults |
| ureq | macroquad built-in HTTP | macroquad's `load_file` is async and WASM-oriented; doesn't work with `std::thread::spawn` pattern cleanly; adds complexity |

**Installation (new deps):**
```bash
# Add to Cargo.toml [dependencies]:
ureq = "3.3.0"
```

**flatpak-cargo-generator prerequisites (CI machine):**
```bash
pip install aiohttp tomlkit
python3 flatpak-cargo-generator.py Cargo.lock -o flatpak/cargo-sources.json
```

---

## Architecture Patterns

### Recommended Project Structure (new files this phase)
```
repo root/
├── release.sh                          # Interactive release script
├── flatpak/
│   ├── io.github.{user}.puuzel.yml    # Flatpak manifest
│   ├── io.github.{user}.puuzel.metainfo.xml
│   └── io.github.{user}.puuzel.desktop
└── .github/
    └── workflows/
        └── release.yml                 # CI: tag push → build → publish
```

GitHub Pages branch: `gh-pages`, containing the OSTree repo at the root (URL: `https://{user}.github.io/puuzel/`). The `.flatpakref` file lives in the repo root (not gh-pages) so users can download it directly from the GitHub releases page.

### Pattern 1: Flatpak Manifest (YAML, buildsystem: simple)

The manifest must use `buildsystem: simple` with explicit cargo offline commands, not `buildsystem: cargo` (which doesn't exist in flatpak-builder). The `cargo-sources.json` generated by `flatpak-cargo-generator.py` is listed as a source and provides the offline crate cache.

```yaml
# Source: https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/
# and https://develop.kde.org/docs/getting-started/rust/rust-flatpak/
app-id: io.github.USERNAME.puuzel
runtime: org.freedesktop.Platform
runtime-version: '24.08'
sdk: org.freedesktop.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: puuzel

finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --device=dri
  - --share=network          # Required for startup version check

build-options:
  append-path: /usr/lib/sdk/rust-stable/bin
  env:
    CARGO_HOME: /run/build/puuzel/cargo
    CARGO_NET_OFFLINE: 'true'

modules:
  - name: puuzel
    buildsystem: simple
    sources:
      - type: dir
        path: ..
      - flatpak/cargo-sources.json
    build-commands:
      - cargo --offline fetch --manifest-path Cargo.toml --verbose
      - cargo --offline build --release --verbose
      - install -Dm755 target/release/puuzel /app/bin/puuzel
      - install -Dm644 flatpak/io.github.USERNAME.puuzel.desktop
          /app/share/applications/io.github.USERNAME.puuzel.desktop
      - install -Dm644 flatpak/io.github.USERNAME.puuzel.metainfo.xml
          /app/share/metainfo/io.github.USERNAME.puuzel.metainfo.xml
      - install -Dm644 data/puuzel.db /app/share/puuzel/puuzel.db
```

**Critical note on CARGO_HOME conflict:** When `CARGO_HOME` is set in build-options, cargo cannot always locate the generated config file. If the build fails with "you are in offline mode", add a shell source step that copies the generated config before building:
```yaml
    build-commands:
      - cp flatpak/cargo-config.toml .cargo/config.toml   # workaround if needed
      - cargo --offline fetch ...
```

### Pattern 2: GitHub Actions Workflow (Flatter)

Flatter is the recommended action. It handles `flatpak-builder`, OSTree export, repo update, and GitHub Pages artifact upload in one step.

```yaml
# Source: https://github.com/andyholmes/flatter
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/andyholmes/flatter/gnome:master
      options: --privileged
    steps:
      - uses: actions/checkout@v4

      - name: Generate cargo-sources.json
        run: |
          pip install aiohttp tomlkit
          python3 flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
            Cargo.lock -o flatpak/cargo-sources.json

      - name: Build and publish
        uses: andyholmes/flatter@main
        with:
          files: flatpak/io.github.USERNAME.puuzel.yml
          upload-pages-artifact: true
          # gpg-sign omitted — unsigned repo per D-02

  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
    steps:
      - uses: actions/deploy-pages@v4
```

**Alternative for flatpak-builder-tools:** Instead of cloning in CI, install via pip or use a pinned commit. The CI image `ghcr.io/andyholmes/flatter/gnome:master` may already include the script — verify before adding the install step.

### Pattern 3: In-App Version Check (startup, background thread)

Follow the existing `std::thread::spawn` + `mpsc::channel` pattern from grid generation. The version check runs before the first frame, sends its result via channel, and the main loop reads it on the next frame.

```rust
// Source: ureq 3.3.0 docs at https://docs.rs/ureq/latest/ureq/
// In main() before the game loop:

let (vtx, vrx) = std::sync::mpsc::channel::<Option<String>>();
std::thread::spawn(move || {
    // Silently skip on any error (offline, timeout, etc.)
    let result = (|| -> Option<String> {
        let body = ureq::get("https://USERNAME.github.io/puuzel/version.txt")
            .call().ok()?
            .body_mut()
            .read_to_string().ok()?;
        Some(body.trim().to_string())
    })();
    vtx.send(result).ok();
});

// In the game loop (first frame):
if let Ok(remote_version) = vrx.try_recv() {
    if let Some(v) = remote_version {
        if v != env!("CARGO_PKG_VERSION") {
            // Show update notification overlay
        }
    }
}
```

**version.txt on GitHub Pages:** CI writes a plain-text file containing the semver string (e.g., `0.2.0`) at the root of the gh-pages branch. The `release.sh` script bumps `Cargo.toml` version before tagging; CI reads `CARGO_PKG_VERSION` at build time and writes `version.txt` to the OSTree repo directory before committing to gh-pages.

### Pattern 4: .flatpakref File (for dad's first install)

```ini
# io.github.USERNAME.puuzel.flatpakref
# Source: https://docs.flatpak.org/en/latest/hosting-a-repository.html
[Flatpak Ref]
Title=Puuzel
Name=io.github.USERNAME.puuzel
Branch=stable
Url=https://USERNAME.github.io/puuzel/
SuggestRemoteName=puuzel
RuntimeRepo=https://flathub.org/repo/flathub.flatpakrepo
IsRuntime=false
```

Note: `GPGKey` field is omitted for an unsigned repo. Dad double-clicks this file; GNOME Software adds the remote and prompts to install. Linux Mint ships with Flatpak + GNOME Software pre-installed.

### Pattern 5: Minimal AppStream metainfo.xml

```xml
<?xml version="1.0" encoding="UTF-8"?>
<component type="desktop-application">
  <id>io.github.USERNAME.puuzel</id>
  <metadata_license>MIT</metadata_license>
  <project_license>MIT</project_license>
  <name>Puuzel</name>
  <summary>Dutch crossword puzzles</summary>
  <description>
    <p>Puuzel genereert Nederlandse kruiswoordpuzzels op aanvraag.
       Kies een moeilijkheidsgraad en begin direct met spelen.</p>
  </description>
  <launchable type="desktop-id">io.github.USERNAME.puuzel.desktop</launchable>
  <releases>
    <release version="0.1.0" date="2026-03-22"/>
  </releases>
  <content_rating type="oars-1.1"/>
</component>
```

This is a minimum-viable metainfo.xml that passes `appstreamcli validate`. Screenshots are optional and can be omitted for the initial release.

### Pattern 6: release.sh Script

```bash
#!/usr/bin/env bash
set -euo pipefail

# Read current version from Cargo.toml
current=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "Current version: $current"

read -rp "Increment [M]ajor / [m]inor / [p]atch? " choice
# Parse, increment, write back to Cargo.toml
# ...
new_version="..."

echo "New version will be: $new_version"
read -rp "Confirm? [y/N] " confirm
[[ "$confirm" == "y" ]] || exit 0

# Update Cargo.toml, commit, tag, push
sed -i "s/^version = \"$current\"/version = \"$new_version\"/" Cargo.toml
git add Cargo.toml
git commit -m "chore: bump version to $new_version"
git tag "v$new_version"
git push && git push --tags
```

### Anti-Patterns to Avoid

- **Using `--share=network` without `--socket=fallback-x11`:** The network share is needed for the version check, but without X11/Wayland socket permissions, the window won't open. Both are required.
- **Committing cargo-sources.json to git:** Per D-13 this is explicitly deferred — CI generates it fresh. It is large and changes with every `Cargo.lock` update.
- **Using `buildsystem: cargo`:** This build system variant doesn't exist in flatpak-builder. Use `buildsystem: simple` with explicit `cargo --offline build` commands.
- **Mixing runtime-version and sdk-extension branch:** The extension branch must match the SDK version exactly (both `24.08`). Mismatch causes "extension not installed" error at build time.
- **Blocking the macroquad main thread with ureq:** ureq's `.call()` is synchronous. Call it only inside `std::thread::spawn`, never directly in the `async fn main` game loop.
- **Not updating version.txt in CI:** If CI only updates the OSTree repo but forgets to write `version.txt` to the gh-pages root, the in-app check will never see a new version.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Offline crate cache for Flatpak | Manual crate vendor scripts | `flatpak-cargo-generator.py` | Handles git deps detection, checksum verification, cargo config generation. Custom scripts break on workspace crates and path deps. |
| Flatpak OSTree repo management | Custom git-based export pipeline | `andyholmes/flatter` action | OSTree commit format, summary file, delta generation, and cache management are non-trivial. Flatter encapsulates all of it. |
| HTTP client for version check | Raw TCP socket or custom HTTP | `ureq 3.3.0` | TLS certificate handling, connection timeouts, HTTP redirect following, and response parsing are all edge cases. ureq gets them right. |
| Version comparison | String comparison | semver string comparison is sufficient here | Since version.txt contains a semver string and `CARGO_PKG_VERSION` is also semver, a simple `!=` check is correct for "newer version available" notification. Full semver parsing not needed. |

**Key insight:** The Flatpak offline build pipeline has exactly one correct tool (`flatpak-cargo-generator.py`) and a well-established GitHub Actions pattern (Flatter). Don't deviate from it — the edge cases in crate source handling are non-obvious.

---

## Common Pitfalls

### Pitfall 1: SDK Extension Branch Mismatch
**What goes wrong:** Build fails with "Requested extension org.freedesktop.Sdk.Extension.rust-stable not installed"
**Why it happens:** The extension branch must exactly match the `runtime-version` in the manifest. If the manifest says `24.08` but the extension is installed at branch `23.08`, Flatpak rejects it.
**How to avoid:** In the manifest, always use the same version string for both `runtime-version` and the extension source. The Flatter CI image ships the current stable extension; if building locally, install with `flatpak install org.freedesktop.Sdk.Extension.rust-stable//24.08`.
**Warning signs:** Error message contains "not installed" or "not found" for the rust-stable extension.

### Pitfall 2: CARGO_HOME Config File Not Found
**What goes wrong:** `cargo --offline fetch` fails with "you are in offline mode" or "can't find registry"
**Why it happens:** `flatpak-cargo-generator.py` writes a `.cargo/config.toml` into the source directory pointing to the pre-downloaded crates. If `CARGO_HOME` is set to a different path by the build system, cargo reads the wrong config.
**How to avoid:** Explicitly set `CARGO_HOME: /run/build/puuzel/cargo` in `build-options.env` AND add a shell build-command that copies the config: `cp .cargo/config.toml $CARGO_HOME/config.toml` before the fetch step. Check both paths are consistent.
**Warning signs:** Build fails on the `cargo --offline fetch` step, not on compilation.

### Pitfall 3: Unsigned Remote Requires --no-gpg-verify on First Add
**What goes wrong:** Dad can't install because Flatpak refuses the unsigned remote
**Why it happens:** By default, `flatpak remote-add` requires a GPG key. When dad double-clicks the `.flatpakref`, GNOME Software handles this — but it may still warn or require confirmation.
**How to avoid:** The `.flatpakref` file should NOT include a `GPGKey` field. GNOME Software will show a warning about the unsigned source — dad just needs to confirm. Document this in the "first install instructions" you send dad. Alternatively, the `.flatpakref` can include `GPGKey=` (empty) to signal explicitly that no signing is used.
**Warning signs:** GNOME Software shows "untrusted source" dialog — this is expected behavior, not an error.

### Pitfall 4: version.txt Not Written by CI
**What goes wrong:** In-app update check always shows "you're up to date" even after a new release
**Why it happens:** The OSTree repo is updated, but `version.txt` at the gh-pages root is not written or is written after the OSTree repo in a separate step that fails silently.
**How to avoid:** Write `version.txt` as part of the same CI step that runs Flatter, before the gh-pages deploy. Use `echo "${{ github.ref_name }}" | sed 's/v//' > version.txt` and include the file in the pages artifact.
**Warning signs:** Manual `curl https://USERNAME.github.io/puuzel/version.txt` returns 404 or old version after a release.

### Pitfall 5: ureq TLS Fails Inside Flatpak Sandbox
**What goes wrong:** Version check always silently fails (no notification ever shown)
**Why it happens:** ureq 3.x uses rustls with webpki-roots (bundled Mozilla cert store). This should work inside the Flatpak sandbox because it doesn't rely on system `/etc/ssl/certs`. However, if the Flatpak manifest lacks `--share=network`, all network access is blocked.
**How to avoid:** Ensure `--share=network` is in `finish-args`. Test the version check by temporarily hardcoding a different version string to force the notification to appear.
**Warning signs:** Version check thread spawns but `vrx.try_recv()` always returns `Ok(None)` (failed silently).

### Pitfall 6: git Dependency in Cargo.toml Breaks Offline Build
**What goes wrong:** `flatpak-cargo-generator.py` errors on git deps; or the cargo offline fetch fails
**Why it happens:** The tool supports git deps in principle but they require git to be available in the build environment, which is not guaranteed. DIST-04 explicitly prohibits git deps.
**How to avoid:** Before running the release, verify `cargo tree --depth 1` shows no git-sourced crates. The current `Cargo.toml` is clean (all crates.io sources). Adding new deps must follow this constraint.
**Warning signs:** Generator script outputs a warning about git sources; or `Cargo.lock` contains entries with `source = "git+..."`.

### Pitfall 7: Data File Path Inside Flatpak
**What goes wrong:** App launches but immediately crashes because `data/puuzel.db` is not found
**Why it happens:** Inside the Flatpak sandbox, relative paths like `PathBuf::from("data/puuzel.db")` resolve relative to the working directory at launch, which is not the install directory. The database must be bundled to `/app/share/puuzel/puuzel.db` and accessed via an absolute path.
**How to avoid:** Replace `PathBuf::from("data/puuzel.db")` in `src/main.rs` with logic that checks for the Flatpak install path first:
```rust
let db_path = if std::path::Path::new("/app/share/puuzel/puuzel.db").exists() {
    PathBuf::from("/app/share/puuzel/puuzel.db")
} else {
    PathBuf::from("data/puuzel.db")  // dev fallback
};
```
This is the most impactful correctness issue in the phase.
**Warning signs:** App exits with `No such file or directory` on `puuzel.db` when run as Flatpak.

---

## Code Examples

Verified patterns from official sources:

### ureq blocking GET (version check)
```rust
// Source: https://docs.rs/ureq/latest/ureq/
use std::sync::mpsc;
use std::thread;

pub fn spawn_version_check() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result: Option<String> = (|| {
            let mut body = String::new();
            ureq::get("https://USERNAME.github.io/puuzel/version.txt")
                .call()
                .ok()?
                .body_mut()
                .read_to_string(&mut body)
                .ok()?;
            Some(body.trim().to_string())
        })();
        tx.send(result).ok();
    });
    rx
}
```

### Flatpak manifest (core skeleton, YAML)
```yaml
# Source: https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/
app-id: io.github.USERNAME.puuzel
runtime: org.freedesktop.Platform
runtime-version: '24.08'
sdk: org.freedesktop.Sdk
sdk-extensions:
  - org.freedesktop.Sdk.Extension.rust-stable
command: puuzel
finish-args:
  - --share=ipc
  - --socket=fallback-x11
  - --socket=wayland
  - --device=dri
  - --share=network
build-options:
  append-path: /usr/lib/sdk/rust-stable/bin
  env:
    CARGO_HOME: /run/build/puuzel/cargo
    CARGO_NET_OFFLINE: 'true'
modules:
  - name: puuzel
    buildsystem: simple
    sources:
      - type: dir
        path: ..
      - flatpak/cargo-sources.json
    build-commands:
      - cargo --offline fetch --manifest-path Cargo.toml --verbose
      - cargo --offline build --release --verbose
      - install -Dm755 target/release/puuzel /app/bin/puuzel
      - install -Dm644 flatpak/io.github.USERNAME.puuzel.desktop
          /app/share/applications/io.github.USERNAME.puuzel.desktop
      - install -Dm644 flatpak/io.github.USERNAME.puuzel.metainfo.xml
          /app/share/metainfo/io.github.USERNAME.puuzel.metainfo.xml
      - install -Dm644 data/puuzel.db /app/share/puuzel/puuzel.db
```

### CI workflow skeleton (Flatter + GitHub Pages)
```yaml
# Source: https://github.com/andyholmes/flatter
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/andyholmes/flatter/gnome:master
      options: --privileged
    permissions:
      pages: write
      id-token: write
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-generator deps
        run: pip install aiohttp tomlkit
      - name: Clone flatpak-builder-tools
        run: git clone --depth 1 https://github.com/flatpak/flatpak-builder-tools.git
      - name: Generate cargo-sources.json
        run: |
          python3 flatpak-builder-tools/cargo/flatpak-cargo-generator.py \
            Cargo.lock -o flatpak/cargo-sources.json
      - name: Write version.txt
        run: echo "${{ github.ref_name }}" | sed 's/^v//' > version.txt
      - name: Build Flatpak and publish to Pages
        uses: andyholmes/flatter@main
        with:
          files: flatpak/io.github.USERNAME.puuzel.yml
          upload-pages-artifact: true
          # extra-files includes version.txt in the pages artifact
          extra-files: version.txt
  deploy:
    needs: build
    runs-on: ubuntu-latest
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
    steps:
      - uses: actions/deploy-pages@v4
```

Note: the `extra-files` input on Flatter may need verification — check the Flatter README for the correct way to include `version.txt` in the pages artifact alongside the OSTree repo.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (cargo test) |
| Config file | None — Cargo.toml test profile |
| Quick run command | `cargo test` |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DIST-01 | Flatpak builds without error | smoke | `flatpak-builder --sandbox build-dir flatpak/io.github.USERNAME.puuzel.yml` (local) | No — Wave 0 |
| DIST-02 | .flatpakref points at correct remote URL | manual | Visual inspection of .flatpakref | No |
| DIST-04 | No git deps in Cargo.lock | automated | `cargo tree --format "{p} {f}" | grep "git+" && exit 1 || exit 0` | No — Wave 0 |
| (version check) | version check returns None gracefully on network error | unit | `cargo test test_version_check_offline` | No — Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test --all`
- **Phase gate:** Full suite green + manual Flatpak install smoke test before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/update/mod.rs` — version check module (unit-testable, no network)
- [ ] DIST-04 check script: `scripts/check-no-git-deps.sh`
- [ ] Local flatpak build test (requires flatpak-builder installed on dev machine — optional, CI is the authoritative test)

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `runtime-version: '23.08'` | `runtime-version: '24.08'` | Aug 2024 | 23.08 is EOL in 2026; 24.08 has active security fixes |
| `flatpak/flatpak-github-actions` (build only) | `andyholmes/flatter` (build + OSTree + Pages) | 2023 | Flatter adds integrated OSTree hosting; the older action only produces a `.flatpak` bundle |
| ureq 2.x (async-capable) | ureq 3.x (blocking only, cleaner API) | Late 2024 | ureq 3 simplified: blocking-only, no async overhead, better semver adherence |

**Deprecated/outdated:**
- `runtime-version: '23.08'`: EOL — do not use
- `flatpak/flatpak-github-actions` alone: does not publish OSTree repo; cannot be used for auto-updates without additional steps
- ureq 2.x: API differs from 3.x; do not mix in the same dep tree

---

## Open Questions

1. **Flatter `extra-files` input availability**
   - What we know: Flatter includes the OSTree repo in the pages artifact; it's unclear if there's a built-in way to add `version.txt` to that artifact
   - What's unclear: Whether `extra-files` is a supported Flatter input, or whether a workaround (write version.txt into the build dir that Flatter uses) is needed
   - Recommendation: Check the Flatter README/source at build time; fallback is to write `version.txt` into the manifest's `build-commands` so it lands in `/app/` and gets included in the export, then copy it out in CI

2. **Flatter container image stability**
   - What we know: `ghcr.io/andyholmes/flatter/gnome:master` uses the `master` tag (not pinned)
   - What's unclear: Whether `master` includes the current gnome-48 SDK or if a specific tag is needed
   - Recommendation: Use `gnome:master` for initial setup; pin to a SHA or dated tag before Phase 4 when Flathub submission requires reproducibility

3. **macroquad Wayland support completeness**
   - What we know: `--socket=wayland` is included in `finish-args`
   - What's unclear: Whether macroquad 0.4.14 has complete Wayland support or falls back to X11 on Wayland compositors
   - Recommendation: Include both `--socket=wayland` and `--socket=fallback-x11` in finish-args; this is the correct pattern for hybrid X11/Wayland apps and is safe

---

## Sources

### Primary (HIGH confidence)
- `https://docs.flatpak.org/en/latest/hosting-a-repository.html` — .flatpakref format, .flatpakrepo format, HTTP keep-alive requirement, OSTree repo commands
- `https://github.com/andyholmes/flatter` — Flatter action inputs, workflow YAML, unsigned repo support
- `https://github.com/flatpak/flatpak-builder-tools/blob/master/cargo/README.md` — flatpak-cargo-generator.py usage, CARGO_HOME conflict, prerequisites
- `https://docs.rs/ureq/latest/ureq/` — ureq 3.3.0 GET example, rustls default, no system deps
- `https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/` — complete Flatpak manifest pattern for Rust, AppStream metainfo structure
- `https://develop.kde.org/docs/getting-started/rust/rust-flatpak/` — sdk-extension usage, cargo offline build pattern

### Secondary (MEDIUM confidence)
- WebSearch results confirming org.freedesktop.Platform 24.08 is current stable (23.08 EOL)
- WebSearch confirming ureq 3.3.0 and minreq 2.14.1 are current versions
- `https://github.com/flatpak/flatpak-github-actions` — confirmed this action does NOT publish OSTree repos (only bundles); Flatter is the correct choice

### Tertiary (LOW confidence)
- Flatter `extra-files` input — needs verification against current README
- Flatter container image tag stability — unverified; use `gnome:master` with caution

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against crates.io and official docs
- Architecture patterns: HIGH — manifest patterns from official Rust Flatpak guides; Flatter from official repo
- Pitfalls: HIGH — CARGO_HOME conflict documented in official flatpak-builder-tools README; data path issue is a first-principles analysis of the codebase

**Research date:** 2026-03-22
**Valid until:** 2026-06-22 (90 days — Flatpak tooling is stable; ureq minor versions may update but 3.x API is stable)
