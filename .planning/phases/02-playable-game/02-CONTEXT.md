# Phase 2: Playable Game - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Rendering, input, puzzle generation, and game flow — the first version you can actually play. User selects difficulty, sees a grid with clue lists, types answers, and gets a congratulations screen when done. No persistence (Phase 3) or distribution (Phase 4).

</domain>

<decisions>
## Implementation Decisions

### Screen layout & grid rendering
- **D-01:** Two-panel layout: crossword grid on the left (~60% width), clue list panel on the right (~40% width)
- **D-02:** Grid cells are square, sized to fill available height. Minimum cell size 32px — target user needs large clickable areas
- **D-03:** Black cells are solid dark gray (#333). White cells have thin borders. Selected cell has a bold blue border. Active word cells have light blue fill
- **D-04:** Letters rendered centered in cells, large and bold. IJ digraph displays as "IJ" in a single cell with slightly compressed font
- **D-05:** Clue numbers rendered small in the top-left corner of numbered cells (traditional crossword style)

### Clue list panel
- **D-06:** Two sections: "Horizontaal" (Across) and "Verticaal" (Down), each scrollable independently
- **D-07:** Active clue is highlighted with blue background and auto-scrolled into view
- **D-08:** Clicking a clue selects that word in the grid and moves cursor to first empty cell
- **D-09:** Font size for clues: 16px minimum — readable without squinting. Clue number bold, clue text regular weight

### Input & navigation
- **D-10:** Click cell to select. Click same cell again to toggle across/down. Type letter to fill and auto-advance to next cell in current direction
- **D-11:** Backspace clears current cell. If already empty, move back one cell and clear that one
- **D-12:** Arrow keys move selection in that direction (skip black cells). Tab/Shift+Tab cycle through clues
- **D-13:** No wrong-letter indication while typing — user only sees if they got it right on completion. This matches newspaper crossword feel (no red highlights mid-solve)

### Difficulty selection & game flow
- **D-14:** Start screen shows "Puuzel" title and three large buttons: "Makkelijk" / "Middel" / "Moeilijk" (Easy/Medium/Hard in Dutch). No other chrome
- **D-15:** Puzzle generation happens on button click. Show "Puzzel wordt gemaakt..." (Puzzle is being generated...) while generating
- **D-16:** When all cells are correctly filled: show "Gefeliciteerd!" (Congratulations!) overlay with a "Nieuwe puzzel" (New puzzle) button that returns to difficulty selection
- **D-17:** Word history tracking: keep last 200 used words in memory to avoid repeats. Resets on app restart (persistence is Phase 3)

### Clue numbering
- **D-18:** Standard crossword numbering: scan left-to-right, top-to-bottom. Each cell that starts an across word or down word (or both) gets the next number. Across and down words sharing a starting cell share the same number.

### Claude's Discretion
- Exact color palette (as long as it's high contrast and readable)
- Scrollbar implementation for clue panel
- Loading indicator style
- Exact font choice (macroquad's built-in or bundled TTF)
- How to handle window resize

</decisions>

<specifics>
## Specific Ideas

- Target user is 70 years old — if anything feels small, cramped, or confusing, it's a bug
- Dutch UI labels throughout: "Horizontaal", "Verticaal", "Makkelijk", "Middel", "Moeilijk", "Gefeliciteerd!", "Nieuwe puzzel"
- No animations or transitions (explicitly out of scope per REQUIREMENTS.md)
- Keyboard-friendly but mouse-primary — this user will mostly click

</specifics>

<canonical_refs>
## Canonical References

No external specs — requirements are fully captured in REQUIREMENTS.md and decisions above.

### Requirements
- `.planning/REQUIREMENTS.md` §Puzzle Generation (PGEN-01 through PGEN-05)
- `.planning/REQUIREMENTS.md` §Grid Interaction (INTR-01 through INTR-09)
- `.planning/REQUIREMENTS.md` §Game Flow (FLOW-01 and FLOW-02 only — FLOW-03/04 are Phase 3)
- `.planning/REQUIREMENTS.md` §Display (DISP-01 through DISP-05)
- `.planning/REQUIREMENTS.md` §Out of Scope — no timer, no hints, no animations, no pencil mode

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/grid/generator.rs`: `generate_grid(conn, difficulty) -> Result<FilledGrid>` — the core generator is done
- `src/grid/types.rs`: `Grid`, `Cell`, `Slot`, `Direction`, `Difficulty`, `DifficultyConfig` — all rendering types exist
- `src/grid/ij.rs`: `tokenize_dutch_word` — needed for display of IJ in cells
- `src/db/mod.rs`: `open_database`, `words_for_length`, `get_clue_for_word` — full DB query layer ready

### Established Patterns
- `DifficultyConfig::easy()` / `::medium()` / `::hard()` — canonical difficulty params
- `LetterToken::IJ` — single-cell IJ display requires special rendering
- `FilledGrid` contains `grid: Grid`, `placed_words: Vec<PlacedWord>` with word_id, word text, slot, clue

### Integration Points
- `main.rs` currently just prints — needs macroquad `#[macroquad::main]` async entry point
- `FilledGrid.placed_words` has all data needed for clue list (word, direction, slot position, clue text)
- `data/puuzel.db` is the database path for `open_database`
- macroquad 0.4 is already in Cargo.toml — rendering, input, and windowing ready to use

</code_context>

<deferred>
## Deferred Ideas

- Puzzle state persistence across restarts — Phase 3
- Thumbs up/down rating persistence — Phase 3
- Flatpak packaging — Phase 4

</deferred>

---

*Phase: 02-playable-game*
*Context gathered: 2026-03-21*
