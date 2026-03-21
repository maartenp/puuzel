# Architecture Research

**Domain:** Native desktop crossword puzzle game (Rust + macroquad)
**Researched:** 2026-03-21
**Confidence:** MEDIUM — macroquad is well-documented; crossword generation patterns are established; Dutch/European grid specifics extrapolated from Wikipedia/community sources

## Standard Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         UI Layer (macroquad)                     │
├───────────────┬──────────────────┬───────────────────────────────┤
│  GridRenderer │   CluePanel      │   MenuScreen / HUD            │
│  (draw cells) │ (draw clue list) │   (lang, difficulty, done)    │
└───────┬───────┴────────┬─────────┴───────────────────────────────┘
        │                │  reads from / sends events to
┌───────▼────────────────▼─────────────────────────────────────────┐
│                     Game State Manager                            │
│  PuzzleState { grid, placed_words, cursor, direction, completed } │
│  UIState { selected_word, clue_lang, screen: GameScreen enum }   │
└──────────────────────────────┬───────────────────────────────────┘
                               │ constructs / queries
          ┌────────────────────┴─────────────────────┐
          │                                           │
┌─────────▼──────────┐                    ┌───────────▼───────────┐
│   Puzzle Generator  │                   │   Word/Clue Database   │
│  (grid layout,      │ ←── word query ── │  (SQLite, bundled)     │
│   word placement,   │                   │  words + 6 clue vars   │
│   constraint solve) │                   │  + feedback ratings    │
└─────────────────────┘                   └───────────────────────┘
                                                      │
                                          ┌───────────▼───────────┐
                                          │   Persistence Layer    │
                                          │  puzzle_state.json     │
                                          │  (serde, user data dir)│
                                          └───────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| GridRenderer | Draw grid cells, letters, highlights, black squares | macroquad draw_rectangle / draw_text calls per frame |
| CluePanel | Render scrollable across/down clue list, highlight active clue | macroquad immediate-mode draw; scroll offset in UIState |
| MenuScreen | New game options (language, difficulty), splash, completion dialog | Separate GameScreen enum variant, drawn in main loop match |
| Game State Manager | Own all mutable game data; process input events; drive transitions | Single `AppState` struct, mutated directly in main loop |
| Puzzle Generator | Produce a valid European-style grid with placed words from word list | Pure function: `fn generate(params, word_pool) -> Puzzle` |
| Word/Clue Database | Store words, six clue variants, feedback ratings, word usage history | SQLite via rusqlite; read-only during generation, write for feedback |
| Persistence Layer | Save/restore current puzzle state across sessions | serde_json to a JSON file in OS user-data directory |

## Recommended Project Structure

```
puuzel/
├── src/
│   ├── main.rs                 # macroquad entry point, main loop, screen dispatch
│   ├── app_state.rs            # AppState, UIState, GameScreen enum, input handling
│   ├── puzzle/
│   │   ├── mod.rs              # Puzzle struct (grid + placed words)
│   │   ├── generator.rs        # Grid layout + word placement algorithm
│   │   ├── grid.rs             # Grid data structure, cell types, coordinate helpers
│   │   └── word.rs             # PlacedWord struct (word, clue_number, pos, dir)
│   ├── database/
│   │   ├── mod.rs              # DB connection, pool
│   │   ├── schema.rs           # Word + clue table definitions
│   │   ├── queries.rs          # Word fetch by difficulty/length, feedback write
│   │   └── word_history.rs     # Recent-word tracking to avoid repetition
│   ├── render/
│   │   ├── mod.rs              # Top-level render dispatch
│   │   ├── grid_renderer.rs    # Cell drawing, highlights, letter display
│   │   ├── clue_panel.rs       # Clue list rendering and scroll
│   │   └── menu.rs             # Start screen, difficulty/language selection
│   ├── persistence.rs          # Save/load puzzle state via serde_json
│   └── config.rs               # Constants: cell size, colors, fonts, difficulty params
├── assets/
│   └── fonts/                  # Bundled font files
├── data/
│   ├── build_db.rs             # Build-time script: import word lists, generate clues
│   └── puuzel.db               # Pre-built SQLite database (committed or generated)
├── build.rs                    # Cargo build script (copy DB to assets if needed)
└── Cargo.toml
```

### Structure Rationale

- **puzzle/:** Generator is a pure computation module with no I/O; easy to test in isolation and fast to call repeatedly if first attempt fails
- **database/:** Isolated behind a query interface so generator receives a `Vec<WordCandidate>` and never touches SQL directly
- **render/:** All macroquad draw calls isolated here; app_state.rs never draws — separation keeps logic testable
- **persistence.rs:** Single file because state save/load is small scope; doesn't need its own module
- **data/:** Build-time tooling separate from runtime code; keeps the binary free of clue-generation logic

## Architectural Patterns

### Pattern 1: Enum-Driven Game Screen State Machine

**What:** A `GameScreen` enum covers all possible states (MainMenu, Playing, Paused, Completed). The main loop matches on it each frame to dispatch rendering and input handling.

**When to use:** Always in macroquad — the engine has no scene graph, so this is the idiomatic replacement.

**Trade-offs:** Simple and explicit. Scales up to ~10 states without friction. Beyond that, state structs per variant become preferable.

**Example:**
```rust
enum GameScreen {
    MainMenu,
    Playing,
    PuzzleComplete,
}

loop {
    match app_state.screen {
        GameScreen::MainMenu => {
            render::menu::draw(&app_state);
            handle_menu_input(&mut app_state);
        }
        GameScreen::Playing => {
            render::grid_renderer::draw(&app_state);
            render::clue_panel::draw(&app_state);
            handle_play_input(&mut app_state);
        }
        GameScreen::PuzzleComplete => {
            render::menu::draw_completion(&app_state);
            handle_completion_input(&mut app_state);
        }
    }
    next_frame().await;
}
```

### Pattern 2: Pure Generator Function

**What:** The puzzle generator takes word candidates as input and returns a completed `Puzzle` struct. No database access, no side effects. Generator = pure function.

**When to use:** Always. Generators that embed I/O are hard to test and can't be retried cheaply if generation fails.

**Trade-offs:** Requires the database layer to pre-fetch a word pool and pass it in. Word pool must be large enough for backtracking to succeed (hundreds of words per difficulty/length bucket).

**Example:**
```rust
pub fn generate(params: &GeneratorParams, word_pool: &[WordCandidate]) -> Result<Puzzle, GeneratorError> {
    let mut grid = Grid::new(params.size);
    place_words_backtracking(&mut grid, word_pool, params)?;
    Ok(Puzzle::from_grid(grid))
}
```

### Pattern 3: Immediate-Mode Input Processing

**What:** macroquad has no event queue — each frame you query input state directly (`is_key_pressed`, `mouse_position`, `is_mouse_button_pressed`). Process all input at the start of each frame before rendering.

**When to use:** Always in macroquad. Don't try to buffer events in a separate thread.

**Trade-offs:** Simple. Cell-click detection requires translating pixel coordinates to grid coordinates every frame — this is cheap and correct.

## Data Flow

### New Puzzle Generation Flow

```
User selects language + difficulty (MenuScreen)
    ↓
AppState::start_new_puzzle(lang, difficulty)
    ↓
database::queries::fetch_word_pool(lang, difficulty, exclude: &recent_words)
    → Vec<WordCandidate> (words + 6 clue variants each)
    ↓
puzzle::generator::generate(params, &word_pool)
    → Puzzle { grid: Grid, placed_words: Vec<PlacedWord> }
    ↓
AppState.puzzle = Some(puzzle)
AppState.screen = GameScreen::Playing
    ↓
persistence::save(&app_state)   // checkpoint for resume
```

### User Input Flow (Playing)

```
Frame start: query macroquad input state
    ↓
if mouse click:
    translate pixel → (col, row)
    if click on grid cell → update cursor, toggle direction or select word
    if click on clue → select word, lock direction, jump cursor to first open cell
    if double-click on word → show rating dialog

if key press:
    if letter key → fill cursor cell, advance cursor in current direction
    if arrow key → move cursor
    if backspace → clear cursor cell, retreat cursor
    ↓
Check puzzle completion: all non-black cells filled correctly?
    yes → screen = GameScreen::PuzzleComplete
    ↓
persistence::save(&app_state)   // save after each keypress (cheap, ~1ms)
```

### Clue Feedback Flow

```
User double-clicks word
    ↓
Render rating overlay (thumbs up / thumbs down)
    ↓
User taps rating
    ↓
database::queries::write_feedback(word_id, clue_variant_id, rating)
    → written to SQLite immediately
```

### Resume Flow (App Launch)

```
main() starts
    ↓
persistence::load() → Option<AppState>
    some → restore puzzle, go to GameScreen::Playing
    none → go to GameScreen::MainMenu
```

### Key Data Flows Summary

1. **Generation:** DB query → word pool → generator → Puzzle struct → AppState
2. **Input:** macroquad input query → coordinate transform → AppState mutation → renderer reads AppState
3. **Persistence:** AppState serialized → JSON file on disk; deserialized on launch
4. **Feedback:** User rating → direct SQLite write (no round-trip through AppState needed)

## Scaling Considerations

This is a single-user desktop application. Traditional scalability concerns (users, servers, load) do not apply. The relevant "scale" concerns are puzzle database size and generation time.

| Concern | At 500 words | At 5,000 words | At 50,000 words |
|---------|--------------|----------------|-----------------|
| DB query speed | Instant | Instant | Instant (SQLite indexed) |
| Generator backtracking | Frequently fails, retry | Occasionally fails | Rarely fails |
| DB file size | ~500KB | ~5MB | ~50MB (acceptable bundled) |
| App startup time | Negligible | Negligible | Negligible |

### Scaling Priorities

1. **First bottleneck:** Generator success rate. With too few words, backtracking fails and generation takes seconds or never completes. Fix: ensure word pool has sufficient words per length bucket before calling generator; tune grid density by difficulty.
2. **Second bottleneck:** Grid size vs. word pool size. A 20x20 grid needs ~30-60 placed words per puzzle. Need at least 3-5x that many eligible words per difficulty to give the constraint solver room to work.

## Anti-Patterns

### Anti-Pattern 1: I/O Inside the Generator

**What people do:** Call the database from inside the placement loop to fetch candidate words on demand.

**Why it's wrong:** Makes the generator stateful, untestable, and slow due to repeated small queries. Also creates coupling that prevents testing generation logic independently.

**Do this instead:** Fetch a word pool upfront (all eligible words for the difficulty/language), pass it into the generator as a slice. The generator's inner loops only touch in-memory data.

### Anti-Pattern 2: Rendering Mixed Into Game Logic

**What people do:** Call `draw_rectangle` or `draw_text` inside the same functions that compute game state (e.g., inside the word placement logic or input handlers).

**Why it's wrong:** macroquad's draw calls are immediate-mode and stateful — mixing them with logic makes both harder to reason about and test. Logic code gets frames-per-second dependencies.

**Do this instead:** Keep AppState pure data. All draw calls live in `render/`. Game logic functions mutate AppState. The main loop separates them: process input → update state → render.

### Anti-Pattern 3: Single Monolithic State Struct

**What people do:** Put cursor position, selected word, scroll offset, language, difficulty, grid data, and word history all in one flat struct.

**Why it's wrong:** Causes excessive borrowing conflicts in Rust (can't have &mut and & simultaneously on different parts). Hard to serialize (persisting UI state is useless). Logic and presentation state mixed together.

**Do this instead:** Split into `PuzzleState` (grid, placed words — needs persistence) and `UIState` (cursor, direction, scroll, selected word — ephemeral). Only serialize `PuzzleState`.

### Anti-Pattern 4: Symmetric Grid Generation for European Style

**What people do:** Enforce 180-degree rotational symmetry on the black square pattern (the American convention).

**Why it's wrong:** Dutch/European crosswords don't use rotational symmetry. Enforcing it artificially constrains placement and produces grids that look wrong to the target user. European grids have ~25% black squares, unchecked letters are acceptable, and 2-letter words may appear.

**Do this instead:** Generate black squares based on density targets per difficulty. Verify connectivity of white squares (flood fill check). Do not require symmetry or fully-checked letters.

### Anti-Pattern 5: Runtime API Calls During Gameplay

**What people do:** Call the Claude API to generate clues when the user starts a new puzzle.

**Why it's wrong:** Requires internet, adds multi-second latency, breaks offline play, and creates API cost per game session.

**Do this instead:** Generate all clues at database build time (build script or separate offline tool). Bundle the pre-generated SQLite database with the app. Gameplay makes zero network calls.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| Claude API (clue generation) | Build-time only — offline script writes to SQLite | Never called at runtime; run once to populate DB |
| Flatpak / OS update | OS-level packaging; no in-app logic needed | App just needs standard file system access to user data dir |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| Generator ↔ Database | Database fetches word pool, passes Vec to generator | Generator has no DB dependency — clean boundary |
| Renderer ↔ AppState | Read-only borrow of AppState each frame | Renderer never mutates state — safe concurrent read |
| Persistence ↔ AppState | Serialize/deserialize PuzzleState only | UIState is ephemeral; don't persist cursor position |
| Input handler ↔ AppState | Exclusive mutable borrow per frame | Process input before rendering; no overlap |
| Feedback ↔ Database | Direct write on user action | Bypasses AppState — feedback is fire-and-forget |

## Build Order Implications

Components have hard dependencies that dictate implementation order:

1. **Grid data structure** (`puzzle/grid.rs`) — everything else depends on it
2. **Word/Clue Database schema + queries** (`database/`) — generator needs it; can stub with hardcoded words initially
3. **Puzzle Generator** (`puzzle/generator.rs`) — depends on Grid; can be developed with a stub word list
4. **AppState + GameScreen state machine** (`app_state.rs`) — the backbone that wires everything
5. **Grid Renderer** (`render/grid_renderer.rs`) — depends on Grid and AppState; needed before the game feels real
6. **Input Handling** (in `app_state.rs`) — depends on Grid for coordinate mapping
7. **Clue Panel** (`render/clue_panel.rs`) — depends on PlacedWord data from generator
8. **Persistence** (`persistence.rs`) — depends on PuzzleState being stable; add after core loop works
9. **Clue Feedback** (`database/queries.rs` write path) — depends on DB schema; low-risk addition late
10. **Word History Tracking** (`database/word_history.rs`) — depends on DB; add after generation works

## Sources

- macroquad state machine documentation: https://docs.rs/macroquad/latest/macroquad/experimental/state_machine/
- macroquad game state chapter: https://mq.agical.se/ch8-game-state.html
- macroquad official site: https://macroquad.rs/
- Crossword as CSP: https://neilagrawal.com/post/implementing-csp-crossword-generation/
- Backtracking for crosswords: https://medium.com/@vanacorec/backtracking-and-crossword-puzzles-4abe195166f9
- Trie-based grid generation: https://www.mdpi.com/1999-4893/15/1/22
- European crossword style rules: https://communicrossings.com/crosswords-terminology-types
- rusqlite Rust cookbook: https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
- Rust game persistence (roguelike tutorial): https://bfnightly.bracketproductions.com/chapter_11.html

---
*Architecture research for: Puuzel — Dutch/English crossword puzzle game (Rust + macroquad)*
*Researched: 2026-03-21*
