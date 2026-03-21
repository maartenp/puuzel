# Project Research Summary

**Project:** Puuzel
**Domain:** Native desktop crossword puzzle game (Dutch/English, European grid style, elderly target user)
**Researched:** 2026-03-21
**Confidence:** MEDIUM (core stack HIGH; grid generation and Dutch-specific conventions MEDIUM)

## Executive Summary

Puuzel is a native desktop crossword puzzle game built in Rust with macroquad, targeting Dutch-speaking elderly users on Linux Mint (Flatpak) and macOS. Research confirms the stack is well-suited: macroquad handles rendering, input, and windowing with minimal dependencies and fast compile times; SQLite via rusqlite provides the indexed word/clue database; serde_json handles state persistence; and Flatpak with flatpak-cargo-generator handles offline distribution. The Dutch/European grid style (unchecked letters allowed, no rotational symmetry, two-letter words permitted, IJ as one cell) is the correct target — it matches what the intended user knows from decades of newspaper puzzles and is not well served by any existing desktop app.

The recommended approach is a phased build: database and grid engine first (both are blocking foundations with no viable stubs), then interactive gameplay loop, then distribution and enrichment features. The puzzle generator must use constraint propagation and pre-indexed word patterns from the start — a naive backtracking algorithm will hang on a 20x20 grid. The clue database must be generated offline via the Claude API with a mandatory answer-verification pass before any clue is stored. All clues must be bundled at build time; runtime API calls are explicitly out of scope and architecturally wrong for Flatpak.

The central risks are: (1) the grid generator failing to scale without proper constraint propagation; (2) the IJ digraph being retrofitted rather than designed in from the first cell; and (3) AI-generated clues containing wrong answers that erode user trust. All three risks are highest-cost to fix late and lowest-cost to design around early. The target user's needs (large fonts, clear direction indicators, high contrast, no anxiety-inducing gamification) are non-negotiable constraints that must inform rendering and interaction decisions from the start, not be bolted on afterward.

## Key Findings

### Recommended Stack

The stack is determined largely by project constraints (Rust, macroquad, Flatpak) and is straightforward. macroquad 0.4.14 is the current stable release; the crate handles OpenGL rendering, input, and windowing for both Linux and macOS without external system dependencies. SQLite (via rusqlite 0.32 with the `bundled` feature) is the right choice for the word/clue database at scale — 50K–300K entries with indexed queries by word length and difficulty, in a single portable file that ships inside the Flatpak bundle. serde + serde_json handles state persistence (JSON is preferable to RON here because it can be inspected and migrated by tooling outside Rust). rand 0.10, directories 6.0, and log/env_logger round out the supporting libraries.

For word databases: OpenTaal (400K+ Dutch words, CC BY / BSD-2-Clause) and SCOWL (public domain English) provide the base word lists. Clues are batch-generated via Claude API at build time at three difficulty levels in both languages, then verified and stored in SQLite. The egui-macroquad crate is a contingency if macroquad's built-in UI proves too limited for the scrollable clue panel.

**Core technologies:**
- Rust (stable 1.8x+): systems language — non-negotiable; single-binary output ideal for Flatpak
- macroquad 0.4.14: rendering, input, windowing — minimal deps, cross-platform, fast compile times
- rusqlite 0.32 (bundled): SQLite word/clue database — indexed queries, single portable file, no system dep
- serde + serde_json 1.x: state persistence — gold standard serialization, JSON enables tooling/migration
- rand 0.10: puzzle generation randomization — API changed in 0.10, do not mix with 0.8
- directories 6.0: XDG-compliant data paths — cross-platform, prevents hardcoded paths
- OpenTaal + SCOWL + Claude API (build-time): word/clue database pipeline — offline, bundled, verified

### Expected Features

**Must have (table stakes for v1):**
- Valid Dutch/European-style grid generation (~20x20) — the core value proposition; nothing else works without it
- Bundled Dutch word+clue database — prerequisite for generation
- Grid rendered on screen with numbered clue list (across/down) — visible, readable, high contrast
- Cell selection with direction toggle (across/down) — core input model
- Keyboard input with auto-advance cursor — filling in letters
- Clue click highlights and selects corresponding word — navigation
- Backspace/delete for error correction — basic correction mechanic
- Puzzle completion detection with congratulations — loop closure
- State persistence (resume on relaunch) — prevents frustration at accidental close
- New puzzle button — start fresh

**Should have (competitive, add after v1 validation):**
- Three difficulty levels — parameterizes generation; add once base generation is proven
- English word+clue database — extend after Dutch is working
- Bilingual clue display with mid-puzzle language toggle — unique differentiator; trivial once data exists
- Clue quality feedback (thumbs up/down) — improvement loop; add once user solves regularly
- Word history tracking — prevents repetition; add once enough puzzles completed
- Flatpak packaging with Flathub distribution — required for target user; add after gameplay is solid

**Defer (v2+):**
- macOS DMG packaging — secondary platform; add after Linux is stable
- English-as-primary-language mode
- Additional grid sizes (small/quick puzzles)

**Anti-features (deliberately excluded):**
- Timer, streaks, statistics dashboard — wrong for a 70-year-old user solving for pleasure
- Social/multiplayer, cloud sync, hints/reveal — wrong scope, wrong platform model
- Animations, pencil mode, in-app clue editing — complexity without value for this user

### Architecture Approach

The architecture is a single-process desktop app with a clear separation between data (AppState/PuzzleState), rendering (macroquad draw calls isolated in render/ modules), and game logic (pure functions). The main loop is a macroquad `loop { next_frame().await }` pattern with an enum-driven game screen state machine (`GameScreen::MainMenu`, `Playing`, `PuzzleComplete`). The puzzle generator is a pure function that receives a pre-fetched word pool and returns a `Puzzle` struct — no I/O inside the generator. The database layer (rusqlite) is the only component that does I/O at runtime; all other I/O is the persistence layer (serde_json to disk). State is split into `PuzzleState` (serialized to disk) and `UIState` (ephemeral, not persisted).

**Major components:**
1. Game State Manager (AppState) — owns all mutable game data, processes input, drives screen transitions
2. Puzzle Generator (puzzle/generator.rs) — pure function: receives word pool, returns valid Puzzle struct
3. Word/Clue Database (database/) — SQLite via rusqlite; read-only during gameplay, write for feedback ratings
4. Grid Renderer (render/grid_renderer.rs) — all macroquad draw calls for the grid; reads AppState, never mutates it
5. Clue Panel (render/clue_panel.rs) — scrollable clue list, synchronized with grid selection
6. Persistence Layer (persistence.rs) — serde_json save/load of PuzzleState to OS user data directory
7. Build-time Database Pipeline (data/) — offline: import word lists, generate clues via Claude API, verify, write SQLite

### Critical Pitfalls

1. **Naive grid generator hangs on 20x20 grids** — use constraint propagation (MCV heuristic, pre-indexed word patterns by length+letter constraints) from day one; never attempt a linear scan; set a hard time limit with restart fallback.

2. **IJ digraph treated as two letters** — design a `DutchLetter` abstraction from the first cell implementation; normalize all Dutch words at database import time; never use raw `.len()` or `.chars().count()` on Dutch words. Retrofitting this later requires rewriting the entire grid engine.

3. **AI-generated clues contain wrong answers** — include a mandatory answer-verification pass in the clue generation pipeline: generate clue, then prompt the model to answer the clue and check that the answer matches the target word. Discard mismatches. Sample 50–100 clues manually before deploying.

4. **Grid produces disconnected white-cell islands** — enforce a flood-fill connectivity check after every grid generation; reject any grid that fails. European grids with high black-square density are especially prone to fragmentation.

5. **Font rendering broken at non-standard DPI on Linux Mint** — derive all font sizes and UI measurements from a single scale factor at startup; never hardcode pixel values. Test explicitly on a real Linux Mint machine at 125% and 150% display scale before declaring rendering complete.

## Implications for Roadmap

Based on research, the dependency graph enforces a clear build order. The grid engine and database are mutually foundational and must come first. Interactive gameplay cannot begin without both. Distribution is a later concern but dependency hygiene (no git-sourced Cargo dependencies) must be enforced from day one to avoid Flatpak build failures.

### Phase 1: Foundation — Grid Engine and Database

**Rationale:** Every other feature depends on a valid grid data structure, a word database, and a generator that can produce connected Dutch/European-style grids. These are the highest-risk items (grid generation is NP-hard; IJ digraph is irreversible if designed wrong) and must be proven correct before anything is built on top of them.

**Delivers:** A tested grid data structure with IJ support; a populated SQLite database with verified Dutch clues at one difficulty level; a generator that reliably produces connected 20x20 grids in under 5 seconds using constraint propagation.

**Addresses (from FEATURES.md):** Dutch word+clue database; valid Dutch/European-style grid generation.

**Avoids (from PITFALLS.md):** Naive generator hanging; IJ treated as two letters; disconnected grid islands.

### Phase 2: Rendering and Core Interaction Loop

**Rationale:** With the grid and database proven, the next dependency layer is making the game visible and interactive. Grid renderer and input handling depend on the grid data structure from Phase 1. This phase produces the first playable version.

**Delivers:** Grid rendered on screen; numbered clue list; cell selection with direction toggle; keyboard input with auto-advance; backspace correction; clue-click navigation; puzzle completion detection with congratulations screen; new puzzle button.

**Uses (from STACK.md):** macroquad 0.4.14 draw calls; immediate-mode input polling.

**Implements (from ARCHITECTURE.md):** GridRenderer, CluePanel, AppState + GameScreen state machine, input handling, enum-driven screen dispatch.

**Avoids (from PITFALLS.md):** Grid interaction UX failing elderly users — strong direction indicator from day one; active clue text displayed prominently; font size minimum 18pt.

### Phase 3: State Persistence and Session Continuity

**Rationale:** The core interaction loop is in place; now make it reliable across sessions. Persistence is a prerequisite for word history tracking (Phase 4) and is a table-stakes feature for any user who closes the app mid-puzzle. It is cleanest to add once PuzzleState is stable.

**Delivers:** Save/load of puzzle state on quit/relaunch using serde_json to OS user data directory (via `directories` crate); format version field included from the start.

**Uses (from STACK.md):** serde + serde_json; directories 6.0.

**Implements (from ARCHITECTURE.md):** persistence.rs; PuzzleState vs. UIState split.

**Avoids (from PITFALLS.md):** State file format incompatibility — include format version field from first implementation.

### Phase 4: Enrichment — Difficulty Levels, Bilingual Clues, Feedback, Word History

**Rationale:** Core gameplay is proven; now add the features that make Puuzel a differentiator rather than a prototype. These features share the persistence layer and database already built. They are parallelizable within this phase.

**Delivers:** Three difficulty levels (parameterized generation and clue selection); English word+clue database; bilingual clue display with mid-puzzle language toggle; clue quality feedback (thumbs up/down stored to SQLite); word history tracking (recent N puzzles, exclusion list fed to generator).

**Uses (from STACK.md):** rusqlite write path for feedback; extended database with 6 clue variants per word (3 difficulties x 2 languages).

**Implements (from ARCHITECTURE.md):** database/word_history.rs; database/queries.rs write path; language/difficulty UIState fields; clue rating overlay.

**Avoids (from PITFALLS.md):** Clue database wrong-difficulty clues — sample review required; easy clues solvable in <3 min, hard clues require >10 min.

### Phase 5: Distribution — Flatpak Packaging and Flathub Submission

**Rationale:** Gameplay is complete. Package and distribute for the target user. Flatpak build tooling is intentionally deferred here because it is a packaging concern, not a gameplay concern — but dependency hygiene (no git-sourced deps) must have been enforced throughout earlier phases for this to succeed cleanly.

**Delivers:** Flatpak bundle built from `cargo-sources.json`; AppStream metainfo.xml; Flathub submission; version string visible in UI.

**Uses (from STACK.md):** flatpak-cargo-generator.py; org.freedesktop.Platform runtime; org.freedesktop.Sdk.Extension.rust-stable.

**Avoids (from PITFALLS.md):** Flatpak offline build failures — all Cargo deps must be crates.io versions (no git deps) throughout the project; cargo-sources.json regenerated after every Cargo.lock change.

### Phase Ordering Rationale

- Database and grid engine come first because they are blocking dependencies with no viable stubs — you cannot test the game loop without them.
- Rendering and interaction come second because they depend on the grid data structure but are independent of persistence and enrichment features.
- Persistence comes third because PuzzleState must be stable before serialization is added; adding it too early risks format churn.
- Enrichment features (difficulty, bilingual, feedback, history) are grouped in Phase 4 because they all extend the existing database and persistence layer without requiring architectural changes.
- Distribution is last because it is a packaging concern and the Flatpak build is cleanest when the app is feature-complete.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 1 (Grid Generation):** Constraint propagation implementation for Dutch/European grids is niche and sparsely documented. The specific data structures for pattern-indexed word lookup (tries vs. HashMaps) need a concrete design decision before coding begins.
- **Phase 1 (Clue Pipeline):** Claude API batch prompt design for three-difficulty, two-language clue generation with verification pass has not been prototyped. Prompt engineering and batch size tuning require experimentation.
- **Phase 5 (Flatpak):** Flathub submission process and review requirements may have changed; verify current requirements against docs.flathub.org before starting.

Phases with standard patterns (research phase likely skippable):
- **Phase 2 (macroquad rendering and input):** Well-documented; macroquad patterns are established and the architecture research provides concrete implementation guidance.
- **Phase 3 (Persistence):** serde_json save/load is straightforward and well-documented; no novel patterns required.
- **Phase 4 (Difficulty/bilingual/feedback):** All extend existing infrastructure; no novel integration required.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Core libraries verified against official docs; macroquad 0.4.14 is current stable; all versions pinned and verified |
| Features | MEDIUM | Grid interaction patterns HIGH; Dutch-specific grid conventions MEDIUM (sparse authoritative sources, inferred from European conventions) |
| Architecture | MEDIUM | macroquad patterns well-documented; crossword generation patterns established from academic sources; Dutch/European grid specifics extrapolated |
| Pitfalls | MEDIUM | Grid generation and IJ pitfalls HIGH (established algorithmic knowledge); macroquad-specific DPI issues MEDIUM; Dutch language specifics MEDIUM |

**Overall confidence:** MEDIUM-HIGH

### Gaps to Address

- **Clue quality at scale:** SCOWL licensing is confirmed public domain but crossword suitability (word length distribution, proper noun filtering, offensive word filtering) is unverified. Run a sample generation before committing to SCOWL as the English base.
- **Generator performance on 20x20 with real dictionary:** The constraint propagation approach is recommended but actual performance on a 20x20 Dutch grid has not been prototyped. Build a proof-of-concept generator early in Phase 1 before committing to the full grid architecture.
- **macroquad egui integration:** The `egui-macroquad` crate is a contingency for the clue panel. Verify version compatibility with macroquad 0.4 before the rendering phase begins — this crate can lag by one minor version.
- **OpenTaal filtering requirements:** The OpenTaal wordlist contains 400K+ words including inflected forms, proper nouns, and rare vocabulary. The filtering pipeline (length 3–15, strip inflections, remove proper nouns) needs to be prototyped before the database size and quality can be confirmed.
- **IJ digraph representation decision:** The specific canonical form (Unicode U+0132 Ĳ vs. two-character "IJ" token) needs to be decided and documented before any word normalization code is written.

## Sources

### Primary (HIGH confidence)
- https://docs.rs/crate/macroquad/latest — macroquad 0.4.14 documentation
- https://macroquad.rs/articles/macroquad-0-4/ — macroquad 0.4 breaking changes
- https://github.com/OpenTaal/opentaal-wordlist — Dutch word list, license verified
- https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo — flatpak-cargo-generator.py
- https://docs.rs/rand/latest/rand/ — rand 0.10.0 API
- https://docs.rs/directories/latest/directories/ — directories 6.0.0
- https://docs.rs/serde_json/latest/serde_json/ — serde_json documentation
- https://docs.flathub.org/docs/for-app-authors/requirements — Flathub submission requirements

### Secondary (MEDIUM confidence)
- https://neilagrawal.com/post/implementing-csp-crossword-generation/ — crossword generation as CSP
- https://www.mdpi.com/1999-4893/15/1/22 — trie-based grid generation, NP-completeness
- https://communicrossings.com/crosswords-terminology-types — European vs. American grid conventions
- https://develop.kde.org/docs/getting-started/rust/rust-flatpak/ — Flatpak + Rust workflow
- https://wordlist.aspell.net/ — SCOWL English word list (licensing confirmed, crossword suitability unverified)
- https://mq.agical.se/ch8-game-state.html — macroquad game state patterns
- https://blog.eyas.sh/2025/12/algorithmic-crosswords/ — practical crossword generation experience
- https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/ — Flatpak Rust packaging

### Tertiary (LOW confidence)
- https://nl.wikipedia.org/wiki/Kruiswoordpuzzel — Dutch crossword conventions (403 error; conventions inferred from other sources)
- https://www.regels.nl/spelletjes/kruiswoord/ — Dutch puzzle conventions (supplementary)

---
*Research completed: 2026-03-21*
*Ready for roadmap: yes*
