# Roadmap: Puuzel

## Overview

Build a native Dutch crossword puzzle game in Rust with macroquad. The work follows a strict dependency order: the grid engine and word database are foundational and blocking; rendering and interaction depend on both; persistence stabilizes the game loop; and distribution caps the work. Four phases deliver a complete, playable, distributable crossword app for a 70-year-old user on Linux Mint.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - Grid engine and Dutch word/clue database — the two blocking dependencies (completed 2026-03-21)
- [ ] **Phase 2: Playable Game** - Rendering, input, puzzle generation, and game flow — the first version you can actually play
- [ ] **Phase 3: Session Continuity** - Puzzle state persistence and clue feedback — the game survives a restart
- [ ] **Phase 4: Distribution** - Flatpak packaging, auto-updates, and macOS build — ships to the target user

## Phase Details

### Phase 1: Foundation
**Goal**: A valid Dutch/European-style crossword grid can be generated and a verified Dutch word+clue database exists bundled with the app
**Depends on**: Nothing (first phase)
**Requirements**: GRID-01, GRID-02, GRID-03, GRID-04, GRID-05, GRID-06, GRID-07, GRID-08, DATA-01, DATA-02, DATA-03, DATA-04, DATA-05, DATA-06
**Success Criteria** (what must be TRUE):
  1. The app generates a 20x20 Dutch/European-style grid with connected white squares and appropriate black-square density, completing in under 10 seconds
  2. The IJ digraph occupies a single cell in any grid that contains a Dutch word with IJ
  3. The bundled SQLite database contains Dutch words with clues at easy, medium, and hard difficulty, each clue having passed an AI self-verification round
  4. The generator respects unchecked letters and permits two-letter words, matching European grid conventions
  5. Grid black-square density and word length distribution visibly differ between easy and hard difficulty
**Plans:** 3/3 plans complete

Plans:
- [x] 01-01-PLAN.md — Project scaffold, core grid types, IJ tokenization, SQLite schema and query layer
- [x] 01-02-PLAN.md — CSP backtracking grid generator with difficulty-dependent density and word selection
- [x] 01-03-PLAN.md — Python word/clue pipeline: OpenTaal filter, Claude Batch API clue generation, database writer

### Phase 2: Playable Game
**Goal**: A human can sit down, start a puzzle at chosen difficulty, fill in answers with keyboard and mouse, and reach a congratulations screen when done
**Depends on**: Phase 1
**Requirements**: PGEN-01, PGEN-02, PGEN-03, PGEN-04, PGEN-05, INTR-01, INTR-02, INTR-03, INTR-04, INTR-05, INTR-06, INTR-07, INTR-08, INTR-09, FLOW-01, FLOW-02, DISP-01, DISP-02, DISP-03, DISP-04, DISP-05
**Success Criteria** (what must be TRUE):
  1. User can select easy, medium, or hard and a new numbered puzzle appears with an across list and a down list
  2. User can click a cell, toggle direction with a second click, type letters that fill cells and auto-advance, and use backspace to correct mistakes
  3. Clicking a clue in the list highlights that word's cells and moves the cursor to the first open cell; the active word is always visibly highlighted
  4. Single-click on a filled word highlights it; double-click opens a thumbs-up/thumbs-down rating prompt
  5. When all cells are correctly filled the app shows a congratulations message and offers to start a new puzzle; fonts are large and readable; grid and clue list fill the screen at high contrast
**Plans**: TBD

### Phase 3: Session Continuity
**Goal**: The puzzle survives an app restart and clue feedback ratings are durably recorded for future quality improvement
**Depends on**: Phase 2
**Requirements**: FLOW-03, FLOW-04
**Success Criteria** (what must be TRUE):
  1. If the user closes the app mid-puzzle and relaunches it, the puzzle resumes exactly where it was left, including filled letters and selected word
  2. Thumbs-up and thumbs-down ratings submitted during a session are written to disk and survive restart
**Plans**: TBD

### Phase 4: Distribution
**Goal**: The app is packaged as a Flatpak that installs and auto-updates on Linux Mint, and also builds and runs on macOS
**Depends on**: Phase 3
**Requirements**: DIST-01, DIST-02, DIST-03, DIST-04
**Success Criteria** (what must be TRUE):
  1. The Flatpak bundle installs on a fresh Linux Mint machine and the app launches without additional dependencies
  2. Running `flatpak update` fetches and applies a new version without user intervention beyond the command
  3. The same codebase compiles and the app runs on macOS without source modifications
  4. All Cargo dependencies resolve from crates.io with no git-sourced deps, confirming Flatpak offline build compatibility
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 3/3 | Complete   | 2026-03-21 |
| 2. Playable Game | 0/TBD | Not started | - |
| 3. Session Continuity | 0/TBD | Not started | - |
| 4. Distribution | 0/TBD | Not started | - |
