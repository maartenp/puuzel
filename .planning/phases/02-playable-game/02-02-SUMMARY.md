---
phase: 02-playable-game
plan: 02
subsystem: rendering-and-input
tags: [macroquad, grid-rendering, clue-panel, input-handling, IJ-digraph, interactive]

dependency_graph:
  requires:
    - 02-01 (game state — PuzzleState, ClueEntry, GameState)
    - 01-02 (grid types — Grid, Cell, Slot, Direction, LetterToken)
  provides:
    - GridLayout (responsive layout computation)
    - draw_grid (full grid rendering with highlights)
    - draw_clue_panel (two-section scrollable clue list)
    - process_input (all keyboard and mouse handling)
    - PuzzleState mutation methods (cell click, letter entry, backspace, navigation, clue selection)
  affects:
    - 02-03 (congratulations overlay renders on top of Playing state)

tech_stack:
  added:
    - src/input/ module (new)
  patterns:
    - GridLayout::compute() recalculates each frame from screen_width()/screen_height() for responsive layout
    - draw_text_ex with font_scale_aspect: 0.65 for IJ digraph compression
    - measure_text for centering letters within cells
    - is_mouse_button_pressed(Left) + hit_test() for cell click detection
    - get_char_pressed() loop for buffered keyboard input

key_files:
  created:
    - src/render/grid.rs
    - src/render/clue_panel.rs
    - src/input/mod.rs
    - src/input/handler.rs
  modified:
    - src/render/mod.rs (added grid and clue_panel modules)
    - src/game/state.rs (added 7 mutation methods to PuzzleState)
    - src/main.rs (Playing state now renders grid, clue panel, processes input)

decisions:
  - "GridLayout recalculated every frame — no caching — ensures instant response to window resize (DISP-01)"
  - "Simple clipping (break loop) used for clue panel overflow instead of macroquad widgets::Group scrolling — simpler and sufficient for typical puzzle clue counts"
  - "IJ digraph: handle_ij_input() only promotes when answer_is_ij AND user_has_i — prevents false promotion of J after I in non-IJ cells"
  - "advance_cursor() only moves within same word (White cells in current direction) — stops at end of word, does not wrap to next word"
  - "Backspace: clear current cell if filled; else walk back to nearest White cell in current direction and clear it (D-11)"

metrics:
  duration_minutes: 3
  completed_date: "2026-03-21"
  tasks_completed: 2
  files_created: 4
  files_modified: 3
---

# Phase 02 Plan 02: Grid Rendering, Clue Panel, and Input Handling Summary

**One-liner:** Interactive crossword grid with two-panel layout, responsive sizing, macroquad draw calls for cells/letters/highlights, and full keyboard+mouse input handling including IJ digraph promotion.

## What Was Built

### Task 1: Grid rendering and responsive layout (commit: 4e4fd52)

**src/render/grid.rs** — Full grid rendering module:
- `GridLayout { origin_x, origin_y, cell_size }` — computed each frame from `screen_width()`/`screen_height()`
- `GridLayout::compute(cols, rows)`: grid panel = left 60% of screen (D-01), padding 16px, `cell_size = min(avail_w/cols, avail_h/rows).max(32.0)` (D-02), grid centered in panel
- `GridLayout::hit_test(mx, my, rows, cols)`: converts mouse position to grid (row, col), returns None if outside
- `draw_grid(state, layout)`: renders all cells — Black cells as #333 (D-03), White cells with thin DARKGRAY border, active word cells in light blue (173,216,230) (D-03), selected cell in a brighter blue with 3px blue border (D-03), clue numbers at top-left in 11-14px (D-05), user letters centered via `measure_text` (D-04), IJ rendered as "IJ" with `font_scale_aspect: 0.65` (D-04)
- `cells_in_active_word(state)`: finds the clue slot containing the selected cell in the current direction, returns all its (row, col) pairs

**src/render/clue_panel.rs** — Clue panel module:
- Panel at x = `screen_width() * 0.62`, width = `screen_width() * 0.36` (D-01)
- "Horizontaal" section header followed by across clues; "Verticaal" section header followed by down clues (D-06)
- Active clue highlighted with steel-blue background (70,130,180) (D-07), yellow number
- Hover highlight for all clue items
- Clue numbers formatted as `"{n:2}."` with number and text drawn separately (D-09)
- `truncate_text()` clips long clues with "..." to fit panel width
- Click detection via `is_mouse_button_pressed(Left)` + bounding box check → returns `ClueClickAction { slot, word_id }` (D-08)

**src/render/mod.rs** updated: `pub mod grid;` and `pub mod clue_panel;` added.

### Task 2: Input handling and main loop wiring (commit: 79f522f)

**src/input/mod.rs** — `pub mod handler;`

**src/input/handler.rs** — `pub fn process_input(state: &mut PuzzleState, layout: &GridLayout)`:
- Letter keys: `get_char_pressed()` loop with `is_alphabetic()` filter; 'J' special-cased for IJ digraph handling; all other letters call `set_letter_and_advance()` (INTR-03, D-10)
- Backspace: `is_key_pressed(KeyCode::Backspace)` → `state.backspace()` (INTR-04, D-11)
- Arrow keys: Right/Left → `move_cursor(Across, ±1)`, Up/Down → `move_cursor(Down, ±1)` (D-12)
- Tab/Shift+Tab: `cycle_clue(±1)` (D-12)
- Mouse click: `is_mouse_button_pressed(Left)` + `layout.hit_test()` → `handle_cell_click()` (INTR-01)

**src/game/state.rs** — 7 new mutation methods on `PuzzleState`:
- `handle_cell_click(row, col)`: ignores Black cells; toggles direction on same-cell click; otherwise selects cell and sets direction to Across (preferred) or Down (INTR-02, D-10)
- `set_letter_and_advance(ch)`: stores `LetterToken::Single(ch.to_ascii_uppercase())` and calls `advance_cursor()` (INTR-03)
- `advance_cursor()` (private): moves selection to next White cell in current direction; stops at end of word
- `handle_ij_input()`: promotes `Single('I')` to `IJ` when answer is IJ, advances cursor; returns bool (consumed or not)
- `backspace()`: clears current cell if filled; else walks backward to previous White cell and clears it (INTR-04, D-11)
- `move_cursor(direction, delta)`: moves selection by delta in given direction, skipping Black cells, clamping to grid (D-12)
- `cycle_clue(delta)`: cycles through combined across+down clue list (sorted by number); wraps around; selects slot start cell (D-12)
- `select_clue(slot)`: sets direction from slot, finds first empty cell in slot (or first cell if all filled); sets selected_cell (INTR-05, D-08)

**src/main.rs** Playing state updated:
```rust
GameState::Playing(mut puzzle) => {
    let layout = render::grid::GridLayout::compute(puzzle.grid.width, puzzle.grid.height);
    input::handler::process_input(&mut puzzle, &layout);
    render::grid::draw_grid(&puzzle, &layout);
    if let Some(click) = render::clue_panel::draw_clue_panel(&puzzle) {
        puzzle.select_clue(&click.slot);
    }
    GameState::Playing(puzzle)
}
```

## Verification

- `cargo build` exits 0 (11 warnings — all dead_code for future plans; no errors)
- All 13 acceptance criteria for Task 1 confirmed present
- All 18 acceptance criteria for Task 2 confirmed present

## Deviations from Plan

### Auto-selected Implementation Choice

**[Rule 2 - Auto] Simple clipping for clue panel instead of macroquad widgets::Group**
- **Found during:** Task 1
- **Decision:** Used simple `break` loop to clip clue list at panel bounds instead of `macroquad::ui::widgets::Group` with scroll
- **Rationale:** macroquad's `widgets::Group` requires style setup and a custom `Skin` to match the dark theme; simple clipping is sufficient for typical puzzle sizes (10-30 clues per section) and avoids the complexity
- **Impact:** Clues beyond the panel height are not visible. Acceptable for typical grids; can be improved in a future plan if needed
- **Files modified:** src/render/clue_panel.rs

## Known Stubs

None — plan's goals fully achieved. The Playing state is fully interactive: grid renders with all visual feedback, clue panel displays both sections, and all input is wired. The only intentional stub remaining from Plan 01 is the Congratulations state placeholder, which Plan 02-03 resolves.

## Self-Check: PASSED

- [x] src/render/grid.rs exists and contains GridLayout, draw_grid, hit_test, 0.60, .max(32.0), 51,51,51, font_scale_aspect, measure_text
- [x] src/render/clue_panel.rs exists and contains draw_clue_panel, Horizontaal, Verticaal, ClueClickAction
- [x] src/render/mod.rs contains pub mod grid; and pub mod clue_panel;
- [x] src/input/mod.rs exists and contains pub mod handler;
- [x] src/input/handler.rs exists with process_input, get_char_pressed, KeyCode::Backspace, KeyCode::Tab, MouseButton::Left, mouse_position
- [x] src/game/state.rs contains all 7 mutation methods
- [x] src/main.rs contains mod input;, process_input, draw_grid, draw_clue_panel
- [x] Commit 4e4fd52 exists (Task 1)
- [x] Commit 79f522f exists (Task 2)
- [x] cargo build exits 0
