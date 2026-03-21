---
phase: 02-playable-game
plan: 01
subsystem: game-state
tags: [game-state, puzzle-state, clue-numbering, word-history, macroquad, menu]

dependency_graph:
  requires:
    - 01-02 (grid generator — FilledGrid, generate_grid)
    - 01-01 (database — open_database, get_clue_for_word)
  provides:
    - GameState enum (drives main loop)
    - PuzzleState (holds grid + user_grid + clue lists)
    - ClueEntry (clue number + text + slot + word_id)
    - assign_clue_numbers (D-18 compliant numbering algorithm)
    - WordHistory (VecDeque<i64> capped at 200)
    - draw_menu_screen (Dutch difficulty buttons)
    - draw_generating_screen (loading screen)
    - macroquad main loop with background generation
  affects:
    - 02-02 (grid rendering plugs into PuzzleState)
    - 02-03 (congratulations overlay uses Congratulations(PuzzleState))

tech_stack:
  added:
    - macroquad::prelude (window, rendering, input)
    - std::sync::mpsc (background thread communication)
    - std::collections::VecDeque (word history)
    - std::collections::HashMap (clue_numbers index)
  patterns:
    - GameState enum state machine driving macroquad frame loop
    - Background thread with mpsc::channel for non-blocking generation
    - try_recv() per frame for polling generation result
    - VecDeque with max-size cap for sliding window history

key_files:
  created:
    - src/game/mod.rs
    - src/game/state.rs
    - src/game/numbering.rs
    - src/game/history.rs
    - src/render/mod.rs
    - src/render/menu.rs
  modified:
    - src/main.rs

decisions:
  - "GameState::Generating uses mpsc::Receiver — GameState cannot derive PartialEq/Clone"
  - "PuzzleState::from_filled_grid called inside background thread where Connection lives (Connection: Send not Sync)"
  - "assign_clue_numbers: words of length 1 never numbered — right/below neighbor must be White"
  - "db_path uses PathBuf::from('data/puuzel.db') matching Phase 1 convention"
  - "exclude HashSet from word_history.recent_ids() is computed but not yet passed to generate_grid (generator API doesn't accept it yet — Phase 3 integration)"

metrics:
  duration_minutes: 15
  completed_date: "2026-03-21"
  tasks_completed: 2
  files_created: 6
  files_modified: 1
---

# Phase 02 Plan 01: Game State Machine and Entry Point Summary

**One-liner:** GameState enum state machine with PuzzleState (from FilledGrid), Dutch difficulty menu, and background puzzle generation via mpsc channels.

## What Was Built

### Task 1: Game types, numbering, word history, and PuzzleState (commit: b103f2c)

**src/game/mod.rs** — Module re-exporting `state`, `numbering`, `history`.

**src/game/numbering.rs** — `assign_clue_numbers(grid: &Grid) -> HashMap<(usize, usize), u32>` implementing D-18: scans left-to-right, top-to-bottom; cells starting an Across word (at left edge or black left neighbor, with White right neighbor) or a Down word (at top edge or black upper neighbor, with White lower neighbor) are assigned the next sequential number. Across and Down words sharing a start cell share the same number. Length-1 words are excluded by the "next cell is White" requirement.

**src/game/history.rs** — `WordHistory` with `VecDeque<i64>` capped at 200 entries. Methods: `new`, `contains`, `add`, `recent_ids`, `add_all`. Evicts oldest entry when cap is exceeded.

**src/game/state.rs** — Core types:
- `ClueEntry { number, text, slot, word_id }` — one entry per clue
- `PuzzleState { grid, user_grid, across_clues, down_clues, selected_cell, selected_direction, clue_numbers, difficulty }` — full puzzle state
- `PuzzleState::from_filled_grid(filled, conn)` — builds PuzzleState from FilledGrid, looks up clue text, sorts clues by number
- `PuzzleState::is_complete()` — checks all White cells with answers against user_grid
- `PuzzleState::active_clue_number()` — walks back from selected_cell to find word start, returns its clue number
- `GameState { DifficultySelection, Generating { rx }, Playing(PuzzleState), Congratulations(PuzzleState) }`

### Task 2: Macroquad main loop, menu screen, background generation (commit: a2e79bc)

**src/render/mod.rs** — Module re-exporting `menu`.

**src/render/menu.rs** — Two functions:
- `draw_menu_screen() -> Option<Difficulty>`: Black background, "Puuzel" title at top third (64px), three 300x70 white buttons centered ("Makkelijk", "Middel", "Moeilijk"), hover highlight, returns clicked difficulty or None
- `draw_generating_screen()`: Black background, "Puzzel wordt gemaakt..." centered (32px)

**src/main.rs** — Full macroquad async main:
- `window_conf()` sets 1280x800, resizable, title "Puuzel"
- `#[macroquad::main(window_conf)]` async fn main
- State machine loop: DifficultySelection → Generating → Playing → Congratulations
- Background thread spawned on difficulty click, opens own DB connection, calls `generate_grid` + `PuzzleState::from_filled_grid`, sends result via mpsc
- `try_recv()` polled each frame (non-blocking)
- `word_history.add_all()` called after successful generation

## Verification

- `cargo test game::` — 14 tests pass (numbering, history, state)
- `cargo build` — compiles without errors (11 warnings, all non-critical: dead_code for future-use variants/methods)
- All acceptance criteria met for both tasks

## Deviations from Plan

None — plan executed exactly as written.

The `exclude` HashSet from `word_history.recent_ids()` is computed but not yet passed to `generate_grid` — the generator API does not accept an exclude list (that integration is Phase 3). This is as-designed per the plan's "word history in memory" specification (PGEN-04 specifies history tracking; the generator exclusion wiring is Phase 3 state persistence work).

## Known Stubs

- `src/main.rs` Playing state: `draw_text("Playing... (rendering coming in Plan 02)", ...)` — intentional placeholder, resolved by Plan 02-02
- `src/main.rs` Congratulations state: `draw_text("Congratulations! (overlay coming in Plan 03)", ...)` — intentional placeholder, resolved by Plan 02-03

These stubs are explicitly documented in the plan as placeholders and do not prevent the plan's goal (launchable app with state machine and background generation) from being achieved.

## Self-Check: PASSED

- [x] src/game/mod.rs exists
- [x] src/game/state.rs exists
- [x] src/game/numbering.rs exists
- [x] src/game/history.rs exists
- [x] src/render/mod.rs exists
- [x] src/render/menu.rs exists
- [x] src/main.rs modified
- [x] Commit b103f2c exists (Task 1)
- [x] Commit a2e79bc exists (Task 2)
- [x] cargo build exits 0
- [x] 14 game:: tests pass
