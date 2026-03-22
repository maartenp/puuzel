<!-- GSD:project-start source:PROJECT.md -->
## Project

**Puuzel**

A native crossword puzzle game for Linux Mint and macOS, built in Rust with macroquad. It generates Dutch and English crossword puzzles in the traditional Dutch/European grid style, with three difficulty levels. Designed for a 70-year-old user who wants a clean, easy-to-use crossword experience on his computer.

**Core Value:** A playable, enjoyable crossword puzzle that generates fresh puzzles on demand — if the puzzle generation and grid interaction work well, everything else is polish.

### Constraints

- **Tech stack**: Rust with macroquad — decided by user, non-negotiable
- **Distribution**: Flatpak for Linux with auto-updates. macOS distribution method TBD (likely DMG or Homebrew)
- **Word/clue data**: Must be bundled with the app or generated at build time — no runtime API calls for clue generation during gameplay
- **Accessibility**: Must be usable by a 70-year-old without tech support. If something is confusing, it's a bug.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | stable (1.8x+) | Systems language | Non-negotiable per project constraints. Gives memory safety, zero-cost abstractions, and single-binary output ideal for Flatpak distribution. |
| macroquad | 0.4.14 | Rendering, input, windowing, audio | The stated requirement. 0.4 is the current stable line (released 2025-03-20). Minimal deps, fast compile times (~16s after `cargo clean`), handles Linux + macOS natively. No external system deps needed. |
| serde + serde_json | serde 1.x, serde_json 1.0.149 | State persistence (save/load puzzle state, clue feedback) | The gold standard for Rust serialization. JSON is the right choice here over RON because the persisted data (word history, clue ratings, save state) may need to be inspected or migrated by tooling. RON has no broad ecosystem outside Rust. TOML lacks enum support needed for game state. |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rand | 0.10.0 | Randomized grid generation, word selection, shuffle | Needed in puzzle generator — `SliceRandom::shuffle()` and `IndexedRandom::choose()` are the key APIs. Use `rand::rng()` for the thread-local RNG. |
| directories | 6.0.0 | Cross-platform XDG-compliant paths for save/config files | Use `ProjectDirs::from("", "", "puuzel")` to find `~/.local/share/puuzel/` on Linux and `~/Library/Application Support/puuzel/` on macOS. Prevents hardcoding paths. |
| log + env_logger | log 0.4.x, env_logger 0.11.x | Debug logging | `log` is the lightweight facade; `env_logger` reads `RUST_LOG=debug` from env. Simpler than `tracing` for a game with no async concerns. Ship with `RUST_LOG=warn` as default. |
### Development Tools
| Tool | Purpose | Notes |
|------|---------|-------|
| flatpak-cargo-generator.py | Pre-generate offline cargo sources for Flatpak builds | Python script from `flatpak/flatpak-builder-tools`. Run against `Cargo.lock` → produces `cargo-sources.json`. Required because Flatpak builds are air-gapped from crates.io. |
| flatpak-builder | Build the Flatpak bundle from manifest | Use with `org.freedesktop.Sdk` runtime + `org.freedesktop.Sdk.Extension.rust-stable`. Set `CARGO_NET_OFFLINE=true` in build-options. |
| cargo vendor | Alternative to flatpak-cargo-generator for vendoring deps | Use if you want vendored deps committed to the release tag rather than a separate cargo-sources.json. More portable but bloats the repo. |
## Word / Clue Databases
### Dutch Words
- 400,000+ words, UTF-8, plain text format, officially recognized by the Dutch Language Union
- License: Creative Commons BY or BSD-2-Clause (dual-licensed) — attribution required, suitable for bundling
- Confidence: HIGH
- Recommendation: Use `wordlist.txt` (base approved words only, ~200K), filter for length 3–15 characters, strip inflected forms if possible. Then batch-generate clues via Claude API at three difficulty levels in Dutch and English. Persist as a bundled SQLite database or a compressed JSON blob.
### English Words
- Use **SCOWL/OPTED** as base word list: SCOWL (Spell Checker Oriented Word Lists) is public domain and available at https://wordlist.aspell.net/ — filter to American English, medium commonality, length 3–15 chars
- Batch-generate clues via Claude API at three difficulty levels in English and Dutch
- Store output in the same bundled database format as Dutch
### Bundled Database Format
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| rusqlite | 0.39.x | Embedded SQLite for word/clue storage and query | Use for the bundled word+clue database. Enables indexed queries like "give me 20 five-letter words at difficulty=easy not in recent_words." Include `bundled` feature flag to statically link SQLite — no system dependency. |
## Flatpak Distribution
### Runtime
### Auto-Updates
### AppStream Metadata
### macOS Distribution
## Installation
# Cargo.toml
# Flatpak tooling (host machine, not inside Flatpak)
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| macroquad 0.4 | Bevy 0.15 | If you need an ECS, 3D, or large team. Bevy has much longer compile times and a steeper learning curve. macroquad's simplicity is correct for this project. |
| macroquad 0.4 | ggez | If you need more mature audio and scene management. ggez is slower to iterate on and less actively maintained than macroquad. |
| macroquad built-in UI | egui via egui-macroquad | egui is richer (tables, text input fields, scroll areas). Consider it if macroquad's built-in UI proves limiting for the clue list panel. The `egui-macroquad` crate bridges the two. |
| serde_json | RON | If save files will be hand-edited by developers frequently. RON looks like Rust syntax and has comment support, but no tooling outside Rust. |
| rusqlite (bundled) | flat JSON word database | For <5K words only. At crossword-viable scale (50K+ words with multiple clues per word) SQLite is dramatically faster to query. |
| flatpak-cargo-generator | cargo vendor | If the release process bakes vendored deps into the source tarball. Either works; generator produces a smaller repo footprint. |
| OpenTaal + AI clues | Existing Dutch crossword database | OpenTaal is used because no open-licensed Dutch crossword clue database was found. Revisit before building the clue pipeline — a licensed dataset saves significant build time. |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| macroquad `megaui` | Deprecated. Was macroquad's original UI, no longer maintained and removed from recent versions. | macroquad's built-in `root_ui()` or `egui-macroquad` |
| `rand` 0.8.x | rand 0.10 is current; 0.8 API diverges on RNG initialization (`thread_rng()` replaced by `rng()`). Mixing versions in the dep tree causes confusion. | `rand = "0.10"` |
| RON for the word database | RON has no tooling for inspecting or migrating large datasets. The clue generation pipeline runs in Python/shell before compile time. | serde_json for interchange, SQLite for runtime queries |
| Runtime HTTP calls to Claude API | Project constraint explicitly forbids runtime API calls during gameplay. All clue data must be bundled at build time. Startup version checks (static file fetch) are permitted. | Offline bundled SQLite database; `version.txt` on GitHub Pages for update checks |
| `directories` < 6.0 | Breaking changes in v3+; API differs for `ProjectDirs`. Always use 6.x. | `directories = "6"` |
| System-provided SQLite (non-bundled rusqlite) | Flatpak sandboxes don't have reliable access to system libraries. The `bundled` feature statically links SQLite into the binary, eliminating the dependency. | `rusqlite = { version = "0.39", features = ["bundled"] }` |
| Full in-app updaters (download + apply) | In-app updaters conflict with the Flatpak trust model. | Rely on `flatpak update`; app notifies user to run it. |
## Stack Patterns by Variant
- Add `egui-macroquad = "0.17"` (verify version against macroquad 0.4 compatibility on crates.io before adding)
- Use egui's `ScrollArea` and `Label` for the scrollable clue list
- Keep rendering logic in macroquad, hand off UI panels to egui
- Compress with zstd at build time; decompress to a temp directory at first launch
- Use the `zstd` Rust crate (0.13.x) for decompression
- Store the decompressed DB in `ProjectDirs.data_dir()` on first run
- Pre-generate a batch of N puzzles at build time and bundle them
- Draw from the pre-generated pool at runtime, regenerating in a background thread when the pool runs low
- macroquad is single-threaded for rendering; use `std::thread::spawn` + `std::sync::mpsc` for background work
## Version Compatibility
| Package | Compatible With | Notes |
|---------|-----------------|-------|
| macroquad 0.4.14 | Rust stable 1.75+ | 0.4 changed shaders to require `ShaderSource` with both GLSL and Metal; avoid raw GLSL strings from 0.3 examples |
| rand 0.10.0 | Rust stable 1.78+ | API change from 0.8: `thread_rng()` → `rng()`, `gen()` → `random()`. Do not mix 0.8 and 0.10 in dep tree. |
| rusqlite 0.39 with `bundled` | SQLite 3.45.x (bundled automatically) | `bundled` feature compiles SQLite from source — increases compile time by ~10s but eliminates all system dependency |
| directories 6.0.0 | All target platforms | v6 stable API; `ProjectDirs::from("", "", "puuzel")` returns correct XDG paths on Linux, NSApplicationSupportDirectory on macOS |
| egui-macroquad | Must match macroquad minor version | Verify crates.io compatibility before pinning — this crate tracks macroquad releases closely and can lag by one minor version |
## Sources
- https://docs.rs/crate/macroquad/latest — macroquad 0.4.14 (latest as of 2026-03-21), HIGH confidence
- https://macroquad.rs/articles/macroquad-0-4/ — macroquad 0.4 changelog and breaking changes, HIGH confidence
- https://github.com/OpenTaal/opentaal-wordlist — Dutch word list, license and format verified, HIGH confidence
- https://develop.kde.org/docs/getting-started/rust/rust-flatpak/ — Flatpak + Rust workflow with flatpak-cargo-generator, MEDIUM confidence (KDE-flavored but process is universal)
- https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo — flatpak-cargo-generator.py official source, HIGH confidence
- https://docs.rs/rand/latest/rand/ — rand 0.10.0 documentation, HIGH confidence
- https://docs.rs/directories/latest/directories/ — directories 6.0.0 documentation, HIGH confidence
- https://docs.rs/serde_json/latest/serde_json/ — serde_json 1.0.149 documentation, HIGH confidence
- https://docs.flathub.org/docs/for-app-authors/requirements — Flathub submission requirements, HIGH confidence
- https://wordlist.aspell.net/ — SCOWL English word list (public domain), MEDIUM confidence (licensing confirmed, crossword suitability unverified)
- WebSearch results for macroquad UI options, rand usage, game logging — MEDIUM confidence, verified against official docs where possible
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
