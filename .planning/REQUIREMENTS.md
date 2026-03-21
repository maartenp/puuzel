# Requirements: Puuzel

**Defined:** 2026-03-21
**Core Value:** A playable, enjoyable crossword puzzle that generates fresh Dutch puzzles on demand

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Grid Engine

- [x] **GRID-01**: App generates valid Dutch/European-style crossword grids (~20x20)
- [x] **GRID-02**: Generated grids have connected white squares (one contiguous region)
- [x] **GRID-03**: IJ digraph is treated as a single cell in the grid
- [x] **GRID-04**: Unchecked letters are permitted (not every letter needs both across and down)
- [x] **GRID-05**: Two-letter words are permitted in the grid
- [x] **GRID-06**: Black square density varies by difficulty (easy = more black squares, hard = fewer)
- [x] **GRID-07**: Word length varies by difficulty (easy = shorter average, hard = longer)
- [x] **GRID-08**: Word commonness varies by difficulty (easy = everyday words, hard = less common)

### Word & Clue Database

- [x] **DATA-01**: Dutch word list sourced and filtered for crossword suitability
- [x] **DATA-02**: AI-generated clue for each word; difficulty derived from word commonness (4-5=easy, 3=medium, 1-2=hard)
- [x] **DATA-03**: Clues are straightforward definitions (not cryptic or wordplay)
- [x] **DATA-04**: Word+clue database bundled with app in SQLite format
- [x] **DATA-05**: Database includes word frequency/commonness metadata for difficulty filtering
- [x] **DATA-06**: AI clue generation includes self-verification pass (best-effort; unverified clues accepted with verified ones preferred at query time)

### Puzzle Generation

- [x] **PGEN-01**: User can start a new puzzle by selecting difficulty level (easy, medium, hard)
- [x] **PGEN-02**: Generator uses constraint satisfaction with backtracking to place words
- [x] **PGEN-03**: Generator produces puzzles in under 10 seconds on typical hardware
- [x] **PGEN-04**: Generator avoids reusing words from the last N puzzles (word history tracking)
- [x] **PGEN-05**: Generated puzzles have numbered clues for across and down words

### Grid Interaction

- [ ] **INTR-01**: User can click a cell to select it
- [ ] **INTR-02**: User can click the same cell again to toggle between across and down direction
- [ ] **INTR-03**: User can type a letter to fill the selected cell and auto-advance to the next cell in current direction
- [ ] **INTR-04**: User can press backspace to clear current cell (if empty, move back and clear)
- [ ] **INTR-05**: User can click a clue in the clue list to highlight that word's cells and select the first open cell
- [ ] **INTR-06**: Clicking a cell belonging to a word locks the direction to that word's direction
- [ ] **INTR-07**: Active word's cells are visually highlighted
- [ ] **INTR-08**: Single-click on a filled word highlights the cells belonging to that clue
- [ ] **INTR-09**: Double-click on a word to rate the clue (thumbs up / thumbs down)

### Game Flow

- [ ] **FLOW-01**: App detects when all cells are correctly filled and shows congratulations
- [ ] **FLOW-02**: After completion, user can start a new puzzle
- [ ] **FLOW-03**: Puzzle state persists across app restarts (quit and resume)
- [ ] **FLOW-04**: Clue feedback ratings (thumbs up/down) are persisted to disk

### Display

- [ ] **DISP-01**: Grid and clue list fill the screen (responsive layout)
- [ ] **DISP-02**: Large readable fonts suitable for elderly users
- [ ] **DISP-03**: High contrast UI (white on black)
- [ ] **DISP-04**: Grid cells are large enough for comfortable reading and clicking
- [ ] **DISP-05**: Clue list is scrollable with current clue visible

### Distribution

- [ ] **DIST-01**: App packaged as Flatpak for Linux Mint
- [ ] **DIST-02**: Flatpak supports auto-updates via standard Flatpak tooling
- [ ] **DIST-03**: App builds and runs on macOS
- [ ] **DIST-04**: All Cargo dependencies use crates.io (no git deps, required for Flatpak offline build)

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### English Language Support

- **LANG-01**: English word+clue database (SCOWL wordlist + AI-generated clues)
- **LANG-02**: User can start a new English puzzle (separate from Dutch)
- **LANG-03**: In-puzzle clue language switching (view clues in Dutch or English regardless of word language)
- **LANG-04**: English clues for Dutch words and Dutch clues for English words (bilingual mode)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Timer / countdown clock | Creates anxiety; wrong for elderly leisure user |
| Streak tracking | Punishes missed days; turns leisure into obligation |
| Puzzle history / statistics | Adds complexity; irrelevant for single-user casual play |
| Social / multiplayer | Single-player product; would require server infrastructure |
| In-app clue editing | Thumbs up/down is sufficient; curation happens offline |
| Cryptic / wordplay clues | Wrong genre for target user |
| NYT-style symmetric grid | Doesn't match Dutch newspaper conventions |
| Cloud save / online sync | Adds complexity and failure modes; local save sufficient |
| Hints / reveal letter | Generation is instant; start a new puzzle instead |
| Pencil mode | Adds UI complexity; target user unlikely to use |
| Animations / transitions | Can confuse elderly users; clean static UI preferred |
| Mobile / tablet | Desktop-only for now |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| GRID-01 | Phase 1 | Complete |
| GRID-02 | Phase 1 | Complete |
| GRID-03 | Phase 1 | Complete |
| GRID-04 | Phase 1 | Complete |
| GRID-05 | Phase 1 | Complete |
| GRID-06 | Phase 1 | Complete |
| GRID-07 | Phase 1 | Complete |
| GRID-08 | Phase 1 | Complete |
| DATA-01 | Phase 1 | Complete |
| DATA-02 | Phase 1 | Complete |
| DATA-03 | Phase 1 | Complete |
| DATA-04 | Phase 1 | Complete |
| DATA-05 | Phase 1 | Complete |
| DATA-06 | Phase 1 | Complete |
| PGEN-01 | Phase 2 | Complete |
| PGEN-02 | Phase 2 | Complete |
| PGEN-03 | Phase 2 | Complete |
| PGEN-04 | Phase 2 | Complete |
| PGEN-05 | Phase 2 | Complete |
| INTR-01 | Phase 2 | Pending |
| INTR-02 | Phase 2 | Pending |
| INTR-03 | Phase 2 | Pending |
| INTR-04 | Phase 2 | Pending |
| INTR-05 | Phase 2 | Pending |
| INTR-06 | Phase 2 | Pending |
| INTR-07 | Phase 2 | Pending |
| INTR-08 | Phase 2 | Pending |
| INTR-09 | Phase 2 | Pending |
| FLOW-01 | Phase 2 | Pending |
| FLOW-02 | Phase 2 | Pending |
| FLOW-03 | Phase 3 | Pending |
| FLOW-04 | Phase 3 | Pending |
| DISP-01 | Phase 2 | Pending |
| DISP-02 | Phase 2 | Pending |
| DISP-03 | Phase 2 | Pending |
| DISP-04 | Phase 2 | Pending |
| DISP-05 | Phase 2 | Pending |
| DIST-01 | Phase 4 | Pending |
| DIST-02 | Phase 4 | Pending |
| DIST-03 | Phase 4 | Pending |
| DIST-04 | Phase 4 | Pending |

**Coverage:**
- v1 requirements: 41 total
- Mapped to phases: 41
- Unmapped: 0

---
*Requirements defined: 2026-03-21*
*Last updated: 2026-03-21 after roadmap creation*
