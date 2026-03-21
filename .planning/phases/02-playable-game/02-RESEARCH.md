# Phase 02: Playable Game - Research

**Researched:** 2026-03-21
**Domain:** macroquad 0.4 rendering, input handling, UI widgets, game state machine, crossword numbering
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Screen layout & grid rendering**
- D-01: Two-panel layout — crossword grid on the left (~60% width), clue list panel on the right (~40% width)
- D-02: Grid cells are square, sized to fill available height. Minimum cell size 32px
- D-03: Black cells are solid dark gray (#333). White cells have thin borders. Selected cell has bold blue border. Active word cells have light blue fill
- D-04: Letters rendered centered in cells, large and bold. IJ digraph displays as "IJ" in a single cell with slightly compressed font_scale_aspect
- D-05: Clue numbers rendered small in the top-left corner of numbered cells

**Clue list panel**
- D-06: Two sections "Horizontaal" (Across) and "Verticaal" (Down), each scrollable independently
- D-07: Active clue highlighted with blue background and auto-scrolled into view
- D-08: Clicking a clue selects that word in the grid and moves cursor to first empty cell
- D-09: Font size for clues: 16px minimum. Clue number bold, clue text regular weight

**Input & navigation**
- D-10: Click cell to select. Click same cell again to toggle across/down. Type letter to fill and auto-advance
- D-11: Backspace clears current cell; if already empty, move back one cell and clear that one
- D-12: Arrow keys move selection in that direction (skip black cells). Tab/Shift+Tab cycle through clues
- D-13: No wrong-letter indication while typing

**Difficulty selection & game flow**
- D-14: Start screen shows "Puuzel" title and three large buttons: "Makkelijk" / "Middel" / "Moeilijk"
- D-15: Show "Puzzel wordt gemaakt..." while generating
- D-16: When all cells correctly filled: "Gefeliciteerd!" overlay with "Nieuwe puzzel" button back to difficulty selection
- D-17: Word history tracking: keep last 200 used words in memory (resets on restart)

**Clue numbering**
- D-18: Standard crossword numbering: scan left-to-right, top-to-bottom. Each cell starting an across or down word (or both) gets the next sequential number

### Claude's Discretion
- Exact color palette (as long as high contrast and readable)
- Scrollbar implementation for clue panel
- Loading indicator style
- Exact font choice (macroquad built-in or bundled TTF)
- How to handle window resize

### Deferred Ideas (OUT OF SCOPE)
- Puzzle state persistence across restarts — Phase 3
- Thumbs up/down rating persistence — Phase 3
- Flatpak packaging — Phase 4
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PGEN-01 | User can start a new puzzle by selecting difficulty level | D-14 menu screen; macroquad button API confirmed |
| PGEN-02 | Generator uses CSP with backtracking | Already implemented in Phase 1; Phase 2 just calls generate_grid() |
| PGEN-03 | Generator produces puzzles in under 10 seconds | Already implemented with 8s timeout; Phase 2 uses background thread pattern |
| PGEN-04 | Generator avoids reusing words from last N puzzles | In-memory HashSet of last 200 word_ids; passed to generate_grid |
| PGEN-05 | Generated puzzles have numbered clues for across and down words | Standard numbering algorithm documented in Architecture Patterns |
| INTR-01 | User can click a cell to select it | mouse_position() + is_mouse_button_pressed(Left) + hit-test against cell rects |
| INTR-02 | User can click same cell again to toggle across/down | Track selected cell; if click == current cell, toggle Direction |
| INTR-03 | User can type a letter to fill and auto-advance | get_char_pressed() + is_alphabetic() filter + advance cursor logic |
| INTR-04 | User can press backspace to clear cell or move back | is_key_pressed(KeyCode::Backspace) handler |
| INTR-05 | User can click a clue to highlight word and select first open cell | Clue panel hit-test; map clue to slot; find first None cell |
| INTR-06 | Clicking a cell belonging to a word locks direction to that word | Click on cell containing a placed word sets direction to that word's Direction |
| INTR-07 | Active word cells are visually highlighted | Render loop checks if each White cell belongs to active slot |
| INTR-08 | Single-click on filled word highlights that word's cells | Same as INTR-06/INTR-07 — no special case needed |
| INTR-09 | Double-click on a word to rate the clue (thumbs up/down) | Track last click time + position; double-click threshold ~300ms; Phase 3 persists rating |
| FLOW-01 | App detects when all cells correctly filled and shows congratulations | After each input: scan all White cells for letter != answer; if none, trigger Congratulations state |
| FLOW-02 | After completion, user can start a new puzzle | "Nieuwe puzzel" button returns to DifficultySelection state |
| DISP-01 | Grid and clue list fill the screen | screen_width() / screen_height() on every frame; responsive layout recalculated each draw |
| DISP-02 | Large readable fonts suitable for elderly users | 32px+ letter cells; 20px clue text; 48px+ title; bundled TTF for consistency |
| DISP-03 | High contrast UI (white on black) | Color palette documented in Architecture Patterns |
| DISP-04 | Grid cells large enough for comfortable reading and clicking | Minimum 32px cell (D-02); at 20x20 grid on 1280x800, ~28px per cell — need to clamp or scale |
| DISP-05 | Clue list is scrollable with current clue visible | macroquad Group widget with scroll_offset tracking; or custom clipping draw |
</phase_requirements>

---

## Summary

This phase builds the entire playable game on top of the completed Phase 1 foundation. The core rendering stack is macroquad 0.4, which is already in Cargo.toml. No new Cargo dependencies are strictly required — the entire game can be built using macroquad's built-in drawing primitives (`draw_rectangle`, `draw_rectangle_lines`, `draw_text_ex`), input functions (`get_char_pressed`, `is_key_pressed`, `is_mouse_button_pressed`, `mouse_position`), and UI system (`root_ui()`, `widgets::Group`).

The biggest architectural decision is whether to use macroquad's built-in `root_ui()` widgets for the clue panel or hand-roll scrollable panel rendering with raw draw calls. The built-in `widgets::Group` supports a scrollable content area (it acts as a mini-window with its own scroll state), which is the recommended approach for the two-section clue list. However, skin customization requires style setup up front.

Puzzle generation must happen off the main render thread because `generate_grid()` can take up to 8 seconds and would freeze the UI. The correct pattern is `std::thread::spawn` + `std::sync::mpsc` channel — open a fresh `rusqlite::Connection` inside the spawned thread (since `Connection` is `Send` but not `Sync`), run the generator, send the `FilledGrid` back over the channel, and poll with `rx.try_recv()` each frame.

**Primary recommendation:** Use macroquad 0.4's native draw calls for all grid rendering, use `widgets::Group` (with custom skin) for the scrollable clue sections, and use `std::thread::spawn + mpsc` for background puzzle generation. No additional Cargo dependencies needed.

---

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| macroquad | 0.4.14 (already in Cargo.toml) | Rendering, input, windowing, UI widgets | Project requirement; already integrated |
| rusqlite (bundled) | 0.39 (already in Cargo.toml) | DB queries for clues during puzzle build | Already integrated; open new connection in generation thread |
| rand | 0.10 (already in Cargo.toml) | Word history shuffle | Already integrated |

### No New Dependencies Required
All functionality needed for Phase 2 is covered by the existing Cargo.toml. No additional crates are needed:

| Need | Solution | Why Not New Dep |
|------|----------|----------------|
| Grid rendering | `macroquad::shapes::draw_rectangle`, `draw_rectangle_lines` | Built into macroquad |
| Text rendering | `macroquad::text::draw_text_ex` with `TextParams` | Built into macroquad |
| Font loading | `load_ttf_font_from_bytes` + `include_bytes!` | Built into macroquad |
| Mouse/keyboard | `macroquad::input::*` | Built into macroquad |
| Scrollable UI | `macroquad::ui::widgets::Group` | Built into macroquad |
| Background thread | `std::thread::spawn` + `std::sync::mpsc` | Rust standard library |

### Optional (Claude's Discretion): Bundled Font
If the built-in macroquad font looks too small or blocky at large sizes, bundle a free TTF. Recommended candidates:
- Noto Sans (Google, OFL license, excellent Dutch character coverage including diacritics)
- DejaVu Sans (public domain, covers full Latin extended)

Embed with `include_bytes!("../assets/fonts/NotoSans-Regular.ttf")` and load with `load_ttf_font_from_bytes`. No Cargo dependency change needed.

**Installation:** No new `cargo add` commands. All deps are present.

---

## Architecture Patterns

### Recommended Project Structure
```
src/
├── main.rs              # #[macroquad::main(conf)] async entry point + game loop
├── game/
│   ├── mod.rs           # pub use
│   ├── state.rs         # GameState enum + PuzzleState struct
│   ├── numbering.rs     # assign_clue_numbers() algorithm
│   └── history.rs       # WordHistory (VecDeque<i64>, last 200)
├── render/
│   ├── mod.rs           # pub use
│   ├── grid.rs          # draw_grid(), draw_cell(), draw_clue_numbers()
│   ├── clue_panel.rs    # draw_clue_panel() with Group scroll
│   ├── menu.rs          # draw_menu_screen()
│   └── overlay.rs       # draw_congratulations_overlay()
├── input/
│   ├── mod.rs           # pub use
│   └── handler.rs       # handle_input() — processes all input events per frame
├── grid/                # Phase 1 — unchanged
└── db/                  # Phase 1 — unchanged
```

### Pattern 1: Game State Machine

```rust
// Source: macroquad book https://mq.agical.se/ch8-game-state.html + project context

#[derive(Debug, Clone, PartialEq)]
enum GameState {
    /// Difficulty selection screen
    DifficultySelection,
    /// Generating puzzle in background; holds channel receiver
    Generating {
        rx: std::sync::mpsc::Receiver<Result<PuzzleState, String>>,
    },
    /// Active play
    Playing(PuzzleState),
    /// All cells filled correctly
    Congratulations(PuzzleState),
}

// Main loop:
#[macroquad::main(conf)]
async fn main() {
    let mut state = GameState::DifficultySelection;
    loop {
        clear_background(BLACK);
        state = update_and_draw(state).await;
        next_frame().await;
    }
}
```

**When to use:** Always. This is the canonical macroquad state machine pattern.

### Pattern 2: Background Puzzle Generation

```rust
// rusqlite::Connection is Send but not Sync.
// Open a new connection inside the spawned thread — do not share across threads.

fn start_generation(difficulty: Difficulty, db_path: PathBuf, history: &WordHistory)
    -> std::sync::mpsc::Receiver<Result<FilledGrid, String>>
{
    let (tx, rx) = std::sync::mpsc::channel();
    let exclude_ids: HashSet<i64> = history.recent_ids().collect();

    std::thread::spawn(move || {
        let conn = match open_database(&db_path) {
            Ok(c) => c,
            Err(e) => { tx.send(Err(e.to_string())).ok(); return; }
        };
        let config = match difficulty {
            Difficulty::Easy => DifficultyConfig::easy(),
            Difficulty::Medium => DifficultyConfig::medium(),
            Difficulty::Hard => DifficultyConfig::hard(),
        };
        let result = generate_grid(&conn, &config)
            .map_err(|e| e.to_string());
        tx.send(result).ok();
    });

    rx
}

// Each frame while in Generating state:
if let Ok(result) = rx.try_recv() {
    state = match result {
        Ok(grid) => GameState::Playing(PuzzleState::from_filled_grid(grid)),
        Err(e) => {
            log::error!("Generation failed: {}", e);
            GameState::DifficultySelection
        }
    };
}
```

**Why `std::thread` not tokio:** macroquad's async is a generator-based coroutine system, not a real async executor. Adding tokio adds ~1.5MB binary size and complex interop for no benefit. `std::thread::spawn` is simpler and correct.

**rusqlite thread constraint:** `Connection` is `Send` (can be moved into a thread) but not `Sync` (cannot be shared). The pattern above moves a path string and opens a fresh connection inside the thread — this is the standard approach.

### Pattern 3: Responsive Grid Layout

```rust
// Called every frame — no caching needed at this scale (20x20 grid)
struct GridLayout {
    origin_x: f32,
    origin_y: f32,
    cell_size: f32,
    grid_width_px: f32,
    grid_height_px: f32,
}

impl GridLayout {
    fn compute(grid_cols: usize, grid_rows: usize) -> Self {
        let sw = screen_width();
        let sh = screen_height();
        let panel_width = sw * 0.60;
        let padding = 16.0;
        let available_w = panel_width - padding * 2.0;
        let available_h = sh - padding * 2.0;
        let cell_by_w = available_w / grid_cols as f32;
        let cell_by_h = available_h / grid_rows as f32;
        let cell_size = cell_by_w.min(cell_by_h).max(32.0); // enforce minimum
        let grid_w = cell_size * grid_cols as f32;
        let grid_h = cell_size * grid_rows as f32;
        GridLayout {
            origin_x: padding + (available_w - grid_w) / 2.0,
            origin_y: padding + (available_h - grid_h) / 2.0,
            cell_size,
            grid_width_px: grid_w,
            grid_height_px: grid_h,
        }
    }

    fn cell_rect(&self, row: usize, col: usize) -> (f32, f32, f32, f32) {
        let x = self.origin_x + col as f32 * self.cell_size;
        let y = self.origin_y + row as f32 * self.cell_size;
        (x, y, self.cell_size, self.cell_size)
    }

    fn hit_test(&self, mx: f32, my: f32, rows: usize, cols: usize) -> Option<(usize, usize)> {
        if mx < self.origin_x || my < self.origin_y { return None; }
        let col = ((mx - self.origin_x) / self.cell_size) as usize;
        let row = ((my - self.origin_y) / self.cell_size) as usize;
        if row < rows && col < cols { Some((row, col)) } else { None }
    }
}
```

### Pattern 4: Clue Numbering Algorithm

Standard newspaper numbering — scan left-to-right, top-to-bottom. A cell gets a number if it starts an Across word (leftmost cell of a horizontal run of 2+ whites) or starts a Down word (topmost cell of a vertical run of 2+ whites), or both. Across and Down words sharing a start cell share the same number.

```rust
// Source: standard crossword convention, D-18

pub fn assign_clue_numbers(grid: &Grid) -> HashMap<(usize, usize), u32> {
    let mut numbers: HashMap<(usize, usize), u32> = HashMap::new();
    let mut next_num: u32 = 1;

    for row in 0..grid.height {
        for col in 0..grid.width {
            if matches!(grid.cells[row][col], Cell::Black) { continue; }

            let starts_across = col == 0
                || matches!(grid.cells[row][col - 1], Cell::Black)
                && col + 1 < grid.width
                && matches!(grid.cells[row][col + 1], Cell::White { .. });

            let starts_down = row == 0
                || matches!(grid.cells[row - 1][col], Cell::Black)
                && row + 1 < grid.height
                && matches!(grid.cells[row + 1][col], Cell::White { .. });

            if starts_across || starts_down {
                numbers.insert((row, col), next_num);
                next_num += 1;
            }
        }
    }
    numbers
}
```

**Note:** A word of length 1 (single unchecked cell) should NOT get a number — the check `col + 1 < grid.width && matches!(...)` handles this for Across; same logic applies Down.

### Pattern 5: Text Rendering with draw_text_ex

```rust
// Source: https://docs.rs/macroquad/0.4.14/macroquad/text/fn.draw_text_ex.html
// TextParams from https://docs.rs/macroquad/0.4.14/macroquad/text/struct.TextParams.html

// Centered letter in a cell
fn draw_cell_letter(token: &LetterToken, cx: f32, cy: f32, cell_size: f32, font: Option<&Font>) {
    let text = match token {
        LetterToken::Single(c) => c.to_string(),
        LetterToken::IJ => "IJ".to_string(),
    };
    let font_size = (cell_size * 0.6) as u16;
    // IJ uses compressed horizontal scale to fit in one cell
    let aspect = if matches!(token, LetterToken::IJ) { 0.65 } else { 1.0 };
    let dims = measure_text(&text, font, font_size, 1.0);
    let x = cx - dims.width * aspect / 2.0;
    let y = cy + dims.height / 2.0;  // macroquad text y is baseline, not top
    draw_text_ex(&text, x, y, TextParams {
        font,
        font_size,
        font_scale: 1.0,
        font_scale_aspect: aspect,
        color: BLACK,
        rotation: 0.0,
    });
}

// Clue number in top-left corner
fn draw_clue_number(number: u32, cell_x: f32, cell_y: f32, font: Option<&Font>) {
    let text = number.to_string();
    draw_text_ex(&text, cell_x + 2.0, cell_y + 10.0, TextParams {
        font,
        font_size: 11,
        color: DARKGRAY,
        ..Default::default()
    });
}
```

**Critical note:** In macroquad, `draw_text_ex`'s `y` parameter is the **baseline** of the text, not the top. To vertically center text in a cell of height `h`, compute `y = cell_top + h/2.0 + dims.height/2.0` where `dims.height` is the cap height from `measure_text`. The `TextDimensions.offset_y` field gives the distance from the baseline to the top of the bounding box, which can help with precise alignment.

### Pattern 6: Input Processing

```rust
// Source: https://docs.rs/macroquad/0.4.14/macroquad/input/index.html

// Called once per frame BEFORE drawing
fn process_input(state: &mut PuzzleState, layout: &GridLayout) {
    // --- Keyboard: letter input ---
    while let Some(ch) = get_char_pressed() {
        if ch.is_alphabetic() {
            let upper = ch.to_ascii_uppercase();
            state.set_letter_and_advance(upper);
        }
    }

    // --- Keyboard: backspace ---
    if is_key_pressed(KeyCode::Backspace) {
        state.backspace();
    }

    // --- Keyboard: arrow keys ---
    if is_key_pressed(KeyCode::Right)  { state.move_cursor(Direction::Across, 1); }
    if is_key_pressed(KeyCode::Left)   { state.move_cursor(Direction::Across, -1); }
    if is_key_pressed(KeyCode::Down)   { state.move_cursor(Direction::Down, 1); }
    if is_key_pressed(KeyCode::Up)     { state.move_cursor(Direction::Down, -1); }

    // --- Keyboard: Tab/Shift+Tab ---
    if is_key_pressed(KeyCode::Tab) {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            state.cycle_clue(-1);
        } else {
            state.cycle_clue(1);
        }
    }

    // --- Mouse: cell click ---
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if let Some((row, col)) = layout.hit_test(mx, my, 20, 20) {
            state.handle_cell_click(row, col);
        }
        // Clue panel clicks handled separately in draw_clue_panel
    }
}
```

**IJ input handling:** When the user types 'I' followed by 'J', the game should treat this as entering the IJ digraph. Recommended: after typing 'I' in a cell, if the next keypress is 'J' and the current cell's answer token is `LetterToken::IJ`, promote the 'I' entry to `LetterToken::IJ`. Alternatively, simply accept any single letter and mark IJ cells as correct only if they have `LetterToken::IJ` placed — then require the user to type the literal character that fills that cell. The simplest UX: since the cell answer is `IJ`, display a hint in the cell (e.g., show nothing but render "IJ" when filled). The user types 'I' and 'J' — the first keystroke 'I' is buffered; on the next keystroke 'J', if the current cell needs IJ, commit `LetterToken::IJ` and advance. Otherwise commit 'I' and keep 'J' in the queue.

### Pattern 7: Scrollable Clue Panel

macroquad's `widgets::Group` widget supports a scrollable content area when the inner content exceeds the group size. However, auto-scroll to the active clue is not built in — you must track a `scroll_offset: f32` and manually set it each frame when the active clue changes.

**Recommended approach for Phase 2:** Custom drawing with `push_scissor` / draw calls is more controllable for auto-scroll behavior. Use `macroquad::ui::widgets::Group` for the scrollable container but track scroll position manually.

```rust
// Clue panel layout
let panel_x = screen_width() * 0.62;
let panel_w = screen_width() * 0.36;
let panel_h = screen_height();

// Across section: top half of panel
let across_h = panel_h * 0.48;
widgets::Group::new(hash!("across"), Vec2::new(panel_w, across_h))
    .position(Vec2::new(panel_x, 30.0))
    .ui(&mut root_ui(), |ui| {
        ui.label(None, "Horizontaal");
        for clue in &state.across_clues {
            let label = format!("{}. {}", clue.number, clue.text);
            if clue.is_active {
                root_ui().push_skin(&highlight_skin);
            }
            if ui.button(None, &label) {
                // handle clue click
            }
            if clue.is_active {
                root_ui().pop_skin();
            }
        }
    });
```

**Limitation of Group auto-scroll:** macroquad's Group widget does not expose an API to programmatically set the scroll offset. For Phase 2 this is acceptable — the user can manually scroll. Auto-scroll to active clue can be approximated by placing the active clue at the top of the rendered list (sort/filter). If true auto-scroll is required, implement a custom draw-with-clipping approach instead.

### Pattern 8: Congratulations Overlay

```rust
fn draw_congratulations(font: Option<&Font>) {
    // Semi-transparent dark overlay
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(),
        Color::new(0.0, 0.0, 0.0, 0.75));

    let sw = screen_width();
    let sh = screen_height();
    let box_w = 400.0;
    let box_h = 200.0;
    let bx = sw / 2.0 - box_w / 2.0;
    let by = sh / 2.0 - box_h / 2.0;

    draw_rectangle(bx, by, box_w, box_h, WHITE);
    draw_rectangle_lines(bx, by, box_w, box_h, 3.0, BLUE);

    // "Gefeliciteerd!" centered
    let title = "Gefeliciteerd!";
    let dims = measure_text(title, font, 36, 1.0);
    draw_text_ex(title, sw / 2.0 - dims.width / 2.0, by + 80.0, TextParams {
        font, font_size: 36, color: DARKGREEN, ..Default::default()
    });

    // "Nieuwe puzzel" button — use root_ui
    root_ui().window(hash!(), Vec2::new(bx, by + 110.0), Vec2::new(box_w, 60.0), |ui| {
        if ui.button(vec2(box_w / 2.0 - 80.0, 10.0), "Nieuwe puzzel") {
            // transition is handled by return value or flag
        }
    });
}
```

### Pattern 9: PuzzleState Data Structure

```rust
pub struct ClueEntry {
    pub number: u32,
    pub text: String,
    pub slot: Slot,
    pub word_id: i64,
    pub is_active: bool,
}

pub struct PuzzleState {
    pub grid: Grid,                          // the 20x20 grid with answer letters
    pub user_grid: Vec<Vec<Option<LetterToken>>>, // what the user has typed (20x20)
    pub across_clues: Vec<ClueEntry>,
    pub down_clues: Vec<ClueEntry>,
    pub selected_cell: Option<(usize, usize)>,
    pub selected_direction: Direction,
    pub clue_numbers: HashMap<(usize, usize), u32>, // cell → clue number
    pub difficulty: Difficulty,
}

impl PuzzleState {
    pub fn from_filled_grid(filled: FilledGrid, conn: &Connection, difficulty_str: &str)
        -> Result<Self, rusqlite::Error>
    {
        // 1. Build user_grid (all None)
        // 2. Assign clue numbers (assign_clue_numbers)
        // 3. Build across_clues and down_clues from slot_words + get_clue_for_word
        // 4. Sort clues by number
    }

    pub fn is_complete(&self) -> bool {
        // Compare user_grid against grid for all White cells
        for r in 0..20 {
            for c in 0..20 {
                if let Cell::White { letter: Some(ref answer) } = self.grid.cells[r][c] {
                    if self.user_grid[r][c].as_ref() != Some(answer) {
                        return false;
                    }
                }
            }
        }
        true
    }
}
```

### Anti-Patterns to Avoid

- **Sharing `rusqlite::Connection` across threads:** `Connection` is `Send` but NOT `Sync`. Never wrap in `Arc<Connection>`. Open a fresh connection in the generation thread.
- **Calling `block_on()` on the macroquad async main thread:** macroquad's async system is not a real executor. Use `std::thread::spawn` for blocking work, poll with `try_recv()` each frame.
- **Using `y` as top-left for `draw_text_ex`:** The `y` parameter is the text **baseline**. Add `dims.offset_y` or `dims.height * 0.75` to convert from top-left to baseline.
- **Re-loading fonts every frame:** Load once at startup, store in a struct, pass references to draw functions.
- **Using `megaui`:** Deprecated and removed from macroquad 0.4. Use `root_ui()` / `widgets::*` instead.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Font rasterization | Custom glyph renderer | macroquad `load_ttf_font_from_bytes` | macroquad uses fontdue internally; handles anti-aliasing, kerning, Unicode |
| Hit testing for cells | Custom spatial index | Simple arithmetic: `(mouse_x - origin_x) / cell_size` | 20x20 grid — O(1) math, no library needed |
| Crossword numbering | Custom algorithm | Pattern 4 above (scan left-to-right, top-to-bottom) | Standard algorithm is ~20 lines; no library needed |
| Background work | Custom thread pool | `std::thread::spawn` + `mpsc::channel` | Single background task (generation); no pool needed |
| Scrollable list | Custom scroll widget from scratch | macroquad `widgets::Group` | Built-in scroll state management |
| Color management | Custom color type | macroquad `Color` + named constants (BLACK, WHITE, BLUE, etc.) | Already in scope via macroquad::prelude |
| Window config | Custom window setup | macroquad `Conf` struct via `#[macroquad::main(conf)]` | `Conf` has all needed fields: title, width, height, resizable |

**Key insight:** macroquad's built-in draw primitives are sufficient for all crossword grid rendering. No scene graph or retained rendering mode is needed — the entire grid is redrawn from scratch each frame (20x20 = 400 cells, trivially fast).

---

## Common Pitfalls

### Pitfall 1: draw_text y-coordinate is baseline, not top-left
**What goes wrong:** Text appears shifted upward, with descenders cut off. Letters look misaligned with their cells.
**Why it happens:** `draw_text_ex(text, x, y, params)` — `y` is the **baseline** of the text, not the top-left corner. The macroquad documentation states this but it's easy to miss.
**How to avoid:** Use `measure_text` to get `TextDimensions`. The `offset_y` field gives the distance from the baseline down to the bottom of the bounding box. To draw centered in a cell of height `h` at top `cell_y`:
```rust
let dims = measure_text(text, font, font_size, 1.0);
let baseline_y = cell_y + (h + dims.height) / 2.0;
draw_text_ex(text, x, baseline_y, params);
```
**Warning signs:** Letters floating above or below the expected center position.

### Pitfall 2: rusqlite::Connection cannot cross thread boundaries via Arc
**What goes wrong:** Compile error "Connection cannot be shared between threads safely" when trying to use `Arc<Connection>`.
**Why it happens:** `Connection` is `Send` (can be moved to another thread) but not `Sync` (cannot be accessed concurrently). `Arc<T>` requires `T: Sync`.
**How to avoid:** Move the database file path (a `PathBuf`) to the spawned thread, open a new `Connection` inside the thread. Do NOT try to share the connection.
**Warning signs:** Rust compiler error mentioning `Sync` and `rusqlite::Connection`.

### Pitfall 3: macroquad UI widgets fighting with custom draw calls
**What goes wrong:** macroquad's `root_ui()` renders at a fixed layer order. Custom `draw_rectangle` calls may appear behind or in front of UI widgets unexpectedly.
**Why it happens:** macroquad renders all `root_ui()` widgets at the end of the frame using default camera coordinates. Custom draws happen when called. Z-ordering is call-order-based.
**How to avoid:** Draw all custom content (grid, letters) first, then call UI widgets. The congratulations overlay using both a `draw_rectangle` (for the dimming layer) and a `root_ui` button should work if the `draw_rectangle` is called before the `root_ui` window in the same frame.
**Warning signs:** UI buttons not clickable because a custom rectangle is drawn on top; or overlay appearing behind the grid.

### Pitfall 4: IJ digraph input UX is confusing
**What goes wrong:** User types 'I', nothing appears (waiting for 'J'), then types 'J', and suddenly two letters appear — or the IJ cell accepts any letter and silently marks it wrong.
**Why it happens:** IJ is a two-keystroke input for a single cell. Naive single-char-per-cell handling breaks Dutch IJ words.
**How to avoid:** Implement a two-stage input buffer for IJ cells: (1) On first keystroke 'I', fill the cell with a pending 'I' display. (2) On next keystroke, if 'J' and cell expects IJ, commit `LetterToken::IJ`. If not 'J', commit `LetterToken::Single('I')` and process the new char normally. The completion check must only mark an IJ cell correct when `user_grid[r][c] == Some(LetterToken::IJ)`.
**Warning signs:** IJ words never shown as correct; or typing 'I' immediately advances past IJ cells.

### Pitfall 5: Panel layout hardcoded pixel values break on different screen sizes
**What goes wrong:** On a 1024x768 screen the clue panel overlaps the grid; on a 4K screen everything is tiny.
**Why it happens:** Using fixed pixel offsets instead of `screen_width()` / `screen_height()` percentages.
**How to avoid:** Recalculate all layout in terms of `screen_width()` and `screen_height()` at the top of each frame. Both values update when the window is resized (macroquad calls the OS resize event). See GridLayout::compute() in Pattern 3.
**Warning signs:** Layout looks fine at 1280x800 but breaks at other resolutions.

### Pitfall 6: Clue numbers on 1-letter slots
**What goes wrong:** Single white cells that are not part of any across or down run get a clue number, producing phantom clues with no word.
**Why it happens:** The naive numbering algorithm assigns numbers to any white cell bordering a black cell, without checking if a run of 2+ whites exists.
**How to avoid:** Only assign a number if the run length in that direction is at least 2. See the `col + 1 < grid.width && matches!(...)` guard in Pattern 4.
**Warning signs:** Clue list shows clues with no corresponding word in the grid.

---

## Code Examples

### macroquad Main Entry Point

```rust
// Source: https://macroquad.rs (official examples)
fn conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Puuzel".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    env_logger::init();
    // load font once
    let font_bytes = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
    let font = macroquad::text::load_ttf_font_from_bytes(font_bytes)
        .expect("failed to load font");

    let db_path = get_db_path(); // uses `directories` crate
    let mut game_state = GameState::DifficultySelection;
    let mut word_history = WordHistory::new(200);

    loop {
        macroquad::window::clear_background(macroquad::color::BLACK);
        game_state = tick(game_state, &font, &db_path, &mut word_history);
        macroquad::window::next_frame().await;
    }
}
```

### Font Loading (Bundled TTF)

```rust
// Source: macroquad text module docs + include_bytes! pattern
// Load at startup, store as Option<Font> in app state
let font = load_ttf_font_from_bytes(include_bytes!("../assets/fonts/NotoSans-Regular.ttf"))
    .expect("font load failed");
// Pass as Option<&Font> to TextParams: font: Some(&font)
```

### Mouse Hit Test

```rust
// Source: macroquad input module docs
use macroquad::input::{mouse_position, is_mouse_button_pressed, MouseButton};

if is_mouse_button_pressed(MouseButton::Left) {
    let (mx, my) = mouse_position();
    // hit test against grid layout
}
```

### Character Input Filter

```rust
// Source: macroquad input module — get_char_pressed consumes from queue
use macroquad::input::get_char_pressed;

while let Some(ch) = get_char_pressed() {
    if ch.is_alphabetic() {
        // process uppercase letter
        let upper = ch.to_ascii_uppercase();
        // ... fill cell logic
    }
    // ignore non-alphabetic (numbers, punctuation, control chars)
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| macroquad `megaui` for UI | `macroquad::ui::root_ui()` + `widgets::*` | macroquad 0.3 → 0.4 | megaui removed; all CLAUDE.md examples use new API |
| `rand::thread_rng()` | `rand::rng()` | rand 0.8 → 0.10 | Already using correct API in Phase 1 |
| GLSL shader strings | `ShaderSource` with both GLSL and Metal | macroquad 0.3 → 0.4 | Not needed for Phase 2 (no custom shaders) |
| `rand::gen()` | `rand::random()` | rand 0.8 → 0.10 | Already handled in Phase 1 |

**Deprecated/outdated:**
- `megaui`: Removed from macroquad 0.4. CLAUDE.md explicitly calls this out. Do not use.
- Raw GLSL string shaders: Changed in 0.4. Not relevant for Phase 2 (no custom shaders needed).
- `directories` < 6.0: Breaking API changes. Already pinned to `"6"` in CLAUDE.md.

---

## Open Questions

1. **Group widget auto-scroll limitation**
   - What we know: macroquad's `widgets::Group` does not expose a public API to set scroll offset programmatically
   - What's unclear: Whether we can approximate auto-scroll by rendering only a window of clues (e.g., the active clue is always item N in the visible range)
   - Recommendation: For Phase 2, accept manual-only scroll. Implement a "jump to active clue" approach: when a new clue is activated, rebuild the clue list with the active entry first, or use `Group::position()` to reposition the entire list. If this proves unsatisfying during testing, implement custom draw-with-clipping for Phase 3.

2. **IJ digraph double-keystroke UX**
   - What we know: The IJ cell contains a `LetterToken::IJ` answer; the user must type both characters
   - What's unclear: Whether a buffered two-keystroke approach or a single-'I'-fills-IJ approach is more natural
   - Recommendation: Implement two-stage buffer (as described in Pitfall 4). Show a grayed 'I' in the cell after first keystroke, commit IJ on 'J'. If UX feels awkward during play-testing, fall back to auto-completing IJ on 'I' keypress (since no Dutch word has 'I' without 'J' in the IJ digraph position).

3. **Cell size on small screens (< 1024px wide)**
   - What we know: Minimum cell size is 32px (D-02); a 20x20 grid at 32px = 640px, plus clue panel ~512px = 1152px total
   - What's unclear: Whether the grid should shrink below 32px on narrow screens or clip/scroll
   - Recommendation: Enforce 32px minimum and let the grid extend slightly into the clue panel space, shrinking the panel. The target user has a desktop — sub-1024px support is not critical for v1.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` (cargo test) |
| Config file | none — standard `cargo test` |
| Quick run command | `cargo test` |
| Full suite command | `cargo test -- --include-ignored` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|--------------|
| PGEN-05 | assign_clue_numbers produces correct numbers for a known grid | unit | `cargo test test_clue_numbering` | ❌ Wave 0 |
| PGEN-04 | WordHistory tracks last N word IDs and excludes them | unit | `cargo test test_word_history` | ❌ Wave 0 |
| FLOW-01 | PuzzleState::is_complete returns true only when all cells match answers | unit | `cargo test test_is_complete` | ❌ Wave 0 |
| INTR-03 | IJ digraph input: typing 'I' then 'J' commits LetterToken::IJ | unit | `cargo test test_ij_input` | ❌ Wave 0 |
| INTR-04 | Backspace: empty cell moves back; filled cell clears | unit | `cargo test test_backspace` | ❌ Wave 0 |
| INTR-01/02 | Cell click + direction toggle logic | unit | `cargo test test_cell_click` | ❌ Wave 0 |
| DISP-01–05 | Visual layout, font size, contrast | manual | play-test on target hardware | manual only |
| PGEN-01–03 | Difficulty selection triggers generation and completes under 10s | manual | play-test (generation already tested in Phase 1) | manual only |

**Note:** Rendering and UI widget tests require a display and are manual-only. The logic that can be unit-tested (numbering, state transitions, input processing) should be extracted into pure functions in `game/` modules.

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test -- --include-ignored`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/game/numbering.rs` — covers PGEN-05: `test_clue_numbering_simple`, `test_clue_numbering_shared_cell`, `test_clue_numbering_no_single_letter_slots`
- [ ] `src/game/history.rs` — covers PGEN-04: `test_word_history_max_size`, `test_word_history_excludes_recent`
- [ ] `src/game/state.rs` — covers FLOW-01: `test_is_complete_all_correct`, `test_is_complete_partial`
- [ ] `src/input/handler.rs` — covers INTR-03/04: `test_ij_input_two_stage`, `test_backspace_empty`, `test_backspace_filled`

---

## Sources

### Primary (HIGH confidence)
- https://docs.rs/macroquad/0.4.14/macroquad/text/index.html — TextParams, draw_text_ex, measure_text, load_ttf_font_from_bytes
- https://docs.rs/macroquad/0.4.14/macroquad/input/index.html — get_char_pressed, is_key_pressed, mouse_position, MouseButton, KeyCode
- https://docs.rs/macroquad/0.4.14/macroquad/window/struct.Conf.html — Conf fields (window_title, window_width, window_height, window_resizable)
- https://docs.rs/macroquad/0.4.14/macroquad/shapes/index.html — draw_rectangle, draw_rectangle_lines, draw_line
- https://docs.rs/macroquad/0.4.14/macroquad/ui/widgets/struct.Group.html — Group widget API (new, position, ui closure)
- https://docs.rs/macroquad/0.4.14/macroquad/text/struct.TextParams.html — TextParams struct fields
- https://docs.rs/macroquad/0.4.14/macroquad/text/fn.measure_text.html — measure_text signature

### Secondary (MEDIUM confidence)
- https://mq.agical.se/ch8-game-state.html — Game state machine pattern with enum + match (Olle Wreede's macroquad book, verified against macroquad source)
- https://mq.agical.se/ch13-menu-ui.html — Button creation, Skin/StyleBuilder, window centering pattern
- https://github.com/not-fl3/macroquad/blob/master/examples/text.rs — Font loading, draw_text_ex, measure_text, get_text_center live example
- https://users.rust-lang.org/t/asynchronously-load-something-in-macroquad/116888 — Background thread + mpsc pattern for off-main-thread work in macroquad
- rusqlite docs (github.com/rusqlite/rusqlite/issues/49) — Connection is Send not Sync; open fresh connection per thread

### Tertiary (LOW confidence — needs play-test validation)
- Group widget scroll auto-positioning: no official API found; custom approach may be needed
- IJ two-stage input UX: designed from first principles; needs play-test validation with target user

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — macroquad 0.4 docs verified; all APIs confirmed via docs.rs
- Architecture patterns (game state, threading, layout): HIGH — verified against official examples and Rust book
- Text rendering (baseline y offset): HIGH — confirmed from draw_text_ex docs and common knowledge
- Clue panel auto-scroll: LOW — Group widget scroll API not publicly documented; workaround needed
- IJ input UX: MEDIUM — logic sound but UX quality needs play-test

**Research date:** 2026-03-21
**Valid until:** 2026-06-21 (macroquad stable API; 90-day window)
