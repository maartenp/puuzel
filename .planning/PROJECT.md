# Puuzel

## What This Is

A native crossword puzzle game for Linux Mint and macOS, built in Rust with macroquad. It generates Dutch and English crossword puzzles in the traditional Dutch/European grid style, with three difficulty levels. Designed for a 70-year-old user who wants a clean, easy-to-use crossword experience on his computer.

## Core Value

A playable, enjoyable crossword puzzle that generates fresh puzzles on demand — if the puzzle generation and grid interaction work well, everything else is polish.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Auto-generated crossword puzzles in Dutch/European grid style (~20x20)
- [ ] Dutch word+clue database (sourced or AI-generated via Claude API)
- [ ] English word+clue database (sourced or AI-generated via Claude API)
- [ ] Three difficulty levels (easy, medium, hard) affecting word commonness, word length, grid density, and clue style
- [ ] Three clue variants per difficulty level × two languages = six clue variants per word
- [ ] Language selection: pick Dutch or English words when starting a new puzzle
- [ ] In-puzzle clue language switching (view clues in Dutch or English regardless of word language)
- [ ] Grid interaction: click cell to select, click again to toggle across/down direction
- [ ] Keyboard input: type letter to fill cell and auto-advance in current direction
- [ ] Clue click: clicking a clue highlights its cells and selects the first open cell, locks direction
- [ ] Cell click within a word locks that word's direction for subsequent typing
- [ ] Single-click on a filled word highlights the cells belonging to that clue
- [ ] Double-click on a word to rate the clue (thumbs up / thumbs down)
- [ ] Clue feedback data persisted for future clue quality improvement
- [ ] Word history tracking: generator avoids reusing words from recent puzzles
- [ ] State persistence: quit and resume current puzzle on relaunch
- [ ] Puzzle completion: congratulations message when puzzle is solved
- [ ] Grid + clue list fills the screen (responsive layout)
- [ ] Clean, high-contrast UI (white on black), large readable fonts
- [ ] Flatpak packaging for Linux Mint distribution
- [ ] Automated update mechanism via Flatpak

### Out of Scope

- Mobile / tablet version — desktop only for now
- Online multiplayer or shared puzzles — single player
- Time tracking, streaks, or puzzle history/statistics — keep it simple
- Cryptic/British-style clues — straightforward definition clues only
- NYT-style symmetric grid — using Dutch/European grid conventions
- In-app clue editing — feedback is thumbs up/down only, curation happens offline

## Context

- **Target user**: The developer's 70-year-old father. UI must prioritize readability, simplicity, and forgiveness. No small text, no complex menus, no confusing states.
- **Dutch crossword conventions**: European-style grids have more black squares than American puzzles, not all letters need to be checked (part of both across and down), and two-letter words may appear. Grid doesn't require rotational symmetry.
- **Word/clue pipeline**: Prefer finding an existing Dutch crossword word+clue database. Fallback: source a Dutch word list, filter for "good crossword words" (appropriate length, no obscure jargon, no offensive content), then batch-generate clues via Claude API at three difficulty levels in two languages. Same approach for English.
- **Clue quality feedback loop**: Thumbs up/down ratings from the user get persisted. Over time this data can be used to filter out bad clues or prioritize good ones.
- **Puzzle generation algorithm**: Must produce valid European-style grids with connected white squares, appropriate black square density varying by difficulty, and proper word placement. Must track previously used words to minimize repetition.
- **Platform targets**: Linux Mint (primary, Flatpak) and macOS (secondary). macroquad supports both.

## Constraints

- **Tech stack**: Rust with macroquad — decided by user, non-negotiable
- **Distribution**: Flatpak for Linux with auto-updates. macOS distribution method TBD (likely DMG or Homebrew)
- **Word/clue data**: Must be bundled with the app or generated at build time — no runtime API calls for clue generation during gameplay
- **Accessibility**: Must be usable by a 70-year-old without tech support. If something is confusing, it's a bug.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust + macroquad | User preference, cross-platform native performance | — Pending |
| Dutch/European grid style | Matches target user's expectations from newspaper puzzles | — Pending |
| Bundled word/clue database | No internet dependency during gameplay | — Pending |
| White-on-black high contrast | Readability for elderly user | — Pending |
| Flatpak distribution | Auto-update support on Linux Mint | — Pending |
| Six clue variants per word | 3 difficulties × 2 languages enables flexible puzzle generation | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-21 after initialization*
