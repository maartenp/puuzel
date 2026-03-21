---
phase: 02-playable-game
plan: 03
subsystem: game-flow-and-overlays
tags: [macroquad, congratulations, completion-detection, double-click, rating-dialog, new-puzzle-flow]

dependency_graph:
  requires:
    - 02-01 (game state — GameState enum, PuzzleState::is_complete, Congratulations variant)
    - 02-02 (rendering — draw_grid, draw_clue_panel, GridLayout, process_input)
  provides:
    - draw_congratulations (Gefeliciteerd! overlay with Nieuwe puzzel button, FLOW-01/02)
    - draw_rating_dialog (double-click clue rating UI stub, INTR-09)
    - InputState (double-click detection state, 300ms threshold)
    - Complete game loop: DifficultySelection -> Generating -> Playing -> Congratulations -> DifficultySelection
  affects:
    - Phase 3 (FLOW-04 rating persistence: word_id + thumbs_up are logged, ready to persist)

tech_stack:
  added: []
  patterns:
    - Manual hit-test button rendering (no macroquad root_ui) for overlay buttons
    - Double-click detection via get_time() delta + same-cell position check
    - is_complete() check per-frame at end of Playing arm to detect puzzle completion

key_files:
  created:
    - src/render/overlay.rs
  modified:
    - src/render/mod.rs (added pub mod overlay;)
    - src/input/handler.rs (added InputState, double-click detection, updated process_input signature)
    - src/main.rs (wired completion check, Congratulations arm, rating dialog, Nieuwe puzzel flow)

decisions:
  - "Manual hit-test buttons for overlays (not macroquad root_ui) — consistent with grid/clue panel approach, avoids UI skin complexity"
  - "Rating dialog: click outside buttons returns Some(false) to dismiss — Phase 3 will distinguish explicit thumbs-down from dismiss"
  - "Double-click threshold: 300ms matching standard OS double-click feel"
  - "is_complete() checked every frame at end of Playing state — negligible cost (O(grid cells))"

metrics:
  duration_minutes: 2
  completed_date: "2026-03-21"
  tasks_completed: 1
  files_created: 1
  files_modified: 3

status: AWAITING_CHECKPOINT
checkpoint_task: "Task 2 — Visual verification of complete game flow"
---

# Phase 02 Plan 03: Congratulations Overlay, Completion Flow, Double-Click Rating Summary

**One-liner:** Congratulations overlay with Gefeliciteerd!/Nieuwe puzzel button, puzzle completion detection via is_complete(), and double-click clue rating dialog (UI stub) closing the full game loop.

**Status: PARTIAL — awaiting human visual verification (Task 2 checkpoint)**

## What Was Built

### Task 1: Congratulations overlay, completion wiring, double-click rating, new puzzle flow (commit: 30d7599)

**src/render/overlay.rs** — Two rendering functions:

- `draw_congratulations() -> bool`: Semi-transparent dark overlay (0.75 alpha) over entire screen. Centered 400x200px white dialog box. "Gefeliciteerd!" at 36px in DARKGREEN (D-16). "Nieuwe puzzel" 200x48px steel-blue button with hover highlight. Returns `true` when the button is clicked — caller transitions to `GameState::DifficultySelection`.

- `draw_rating_dialog(clue_text: &str) -> Option<bool>`: Small 380x160px dialog with clue text, "Goed" (green) and "Slecht" (red) rating buttons. Returns `Some(true)` / `Some(false)` on button click, `None` while open. Clicking outside buttons dismisses with `Some(false)`. Phase 3 (FLOW-04) will add persistence.

**src/render/mod.rs** — Added `pub mod overlay;`.

**src/input/handler.rs** — New types and updated function:

- `pub struct RatingContext { word_id: i64, clue_text: String }` — payload for an active rating dialog
- `pub struct InputState { last_click_time: f64, last_click_pos: Option<(usize, usize)>, rating_dialog: Option<RatingContext> }` — double-click tracking state
- `InputState::new() -> Self` — initializes with sentinel `last_click_time: -1.0`
- `process_input(state, layout, input_state)` — extended signature accepts `&mut InputState`. On left mouse click: if same cell clicked within 300ms (`DOUBLE_CLICK_THRESHOLD`), it's a double-click: finds the clue for that cell via `find_clue_at_cell()` and populates `input_state.rating_dialog`. Otherwise normal `handle_cell_click()`.
- `find_clue_at_cell()` / `find_clue_in_direction()` — helpers to look up which ClueEntry covers a given (row, col) in the selected (or fallback) direction.

**src/main.rs** — Full game loop wired:

- `let mut input_state = input::handler::InputState::new();` before the loop
- Playing arm: passes `&mut input_state` to `process_input`; after drawing, shows `draw_rating_dialog` if `input_state.rating_dialog` is `Some`; calls `puzzle.is_complete()` each frame; transitions to `GameState::Congratulations(puzzle)` on completion (FLOW-01)
- Congratulations arm: renders grid + clue panel underneath; shows `draw_congratulations()` overlay; on click → `GameState::DifficultySelection` (FLOW-02)

## Verification

- `cargo build` exits 0 (9 warnings — all pre-existing dead_code; no new warnings from Plan 03 changes)
- All 14 acceptance criteria confirmed present in source files

## Deviations from Plan

None — plan executed exactly as written.

The one implementation note: the `draw_rating_dialog` dismiss-on-click-outside behavior returns `Some(false)` rather than a dedicated `None` (which means "still open"). This is acceptable for Phase 2 since ratings aren't persisted; Phase 3 can refine the return type to distinguish "thumbs down" from "dismissed".

## Known Stubs

- `draw_rating_dialog`: Rating result is logged (`log::info!`) but not persisted — intentional stub for Phase 3 (FLOW-04 clue feedback persistence).

## Self-Check: PARTIAL (pending Task 2 checkpoint)

- [x] src/render/overlay.rs exists with draw_congratulations and draw_rating_dialog
- [x] src/render/mod.rs contains pub mod overlay;
- [x] src/input/handler.rs contains InputState, last_click_time, rating_dialog
- [x] src/main.rs contains puzzle.is_complete(), GameState::Congratulations, draw_congratulations(), draw_rating_dialog, GameState::DifficultySelection in Congratulations arm
- [x] Commit 30d7599 exists (Task 1)
- [x] cargo build exits 0
- [ ] Task 2: Human visual verification (pending checkpoint)
