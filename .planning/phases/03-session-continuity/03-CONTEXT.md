# Phase 3: Session Continuity - Context

**Gathered:** 2026-03-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Puzzle state persistence and clue feedback — the game survives a restart. Users can save multiple unfinished puzzles and resume them later. Clue ratings are durably recorded. Word history persists with a decay model so the generator avoids recent repeats while never blocking puzzle creation.

</domain>

<decisions>
## Implementation Decisions

### User data storage
- **D-01:** All user data stored in a separate `user.db` (SQLite) in the platform data directory (`directories` crate → `ProjectDirs.data_dir()`)
- **D-02:** The bundled `puuzel.db` remains read-only — never written to at runtime
- **D-03:** `user.db` uses `ATTACH DATABASE` to join against `puuzel.db` when queries need both (e.g., filtering out thumbs-down clues during generation)

### Puzzle save/load behavior
- **D-04:** Auto-save after every letter typed — write `PuzzleState` to `user.db`
- **D-05:** Keep a backup: write to working row, keep previous good state as backup (recover from corruption)
- **D-06:** Multiple unfinished puzzles supported — no limit on saved puzzles
- **D-07:** Completed puzzles are removed from the saved list automatically
- **D-08:** Save file for current puzzle kept until a new puzzle finishes generating (not deleted on "Nieuwe puzzel" click)

### Menu flow changes
- **D-09:** App always opens to the main menu (never auto-resumes)
- **D-10:** Main menu adds an "Onvoltooide puzzels" button that opens a list of saved unfinished puzzles
- **D-11:** Each saved puzzle shows difficulty and completion percentage; has resume and delete actions
- **D-12:** Delete requires confirmation dialog: "Weet je het zeker?"
- **D-13:** Difficulty selection screen shows "Terug naar puzzel" button when a puzzle is currently active (lets user cancel out before generating)
- **D-14:** Clicking "Nieuwe puzzel" while mid-puzzle silently saves current puzzle to the unfinished list and goes to difficulty selection

### Clue feedback persistence
- **D-15:** Both thumbs-up and thumbs-down ratings are recorded in `user.db`
- **D-16:** Ratings written to SQLite immediately on each thumbs-up/down click
- **D-17:** Thumbs-down means "never show this clue again" — permanent exclusion (D-21 from Phase 1)
- **D-18:** Word-level ratings (D-23 from Phase 1) are abandoned — clue-level only

### Word history with decay
- **D-19:** Each word use stored in `user.db` as `(word_id, puzzle_number)` — no cap on history size
- **D-20:** Puzzle number is a monotonically increasing counter stored in `user.db`
- **D-21:** Two-tier exclusion model:
  - **Tier 1 (hard exclude):** Words from the last N puzzles are completely excluded from candidate lists
  - **Tier 2 (prefer least-recent):** When all candidates at a given length have been used, rank by least-recently-used and pick from the oldest
- **D-22:** Decay is best-effort — never blocks puzzle generation. If the generator can't fill a slot, history constraints are relaxed for that slot

### Claude's Discretion
- Exact value of N for tier-1 hard exclusion (tunable constant)
- SQLite schema design for `user.db` (puzzle saves, ratings, word history tables)
- Serialization format for `PuzzleState` in the database (JSON blob vs normalized tables)
- Backup rotation strategy details
- How to compute/display completion percentage for saved puzzles

</decisions>

<specifics>
## Specific Ideas

- Dutch UI labels: "Onvoltooide puzzels", "Terug naar puzzel", "Weet je het zeker?", "Nieuwe puzzel"
- The decay model exists because common crossword lengths (4-6 letters) have a limited pool of good words — pure exclusion would exhaust them quickly
- Target user does ~1 puzzle/day — history growth is trivial (~18K entries/year)

</specifics>

<canonical_refs>
## Canonical References

### Requirements
- `.planning/REQUIREMENTS.md` §Game Flow — FLOW-03 (puzzle state persistence), FLOW-04 (clue feedback persistence)
- `.planning/REQUIREMENTS.md` §Out of Scope — no cloud save, no puzzle history/statistics

### Prior phase decisions
- `.planning/phases/01-foundation/1-CONTEXT.md` §Clue feedback system — D-20 through D-23 (rating mechanics, thumbs-down = permanent blacklist). Note: D-23 (word-level ratings) is now abandoned per D-18 above
- `.planning/phases/01-foundation/1-CONTEXT.md` §Clue style per difficulty — D-17 through D-19 (clue generation context)
- `.planning/phases/02-playable-game/02-CONTEXT.md` §Difficulty selection & game flow — D-14 through D-17 (menu structure this phase extends)

### Tech stack
- `CLAUDE.md` §Technology Stack — `directories` crate 6.0 for cross-platform data paths, `rusqlite` with bundled feature, `serde_json` for serialization

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/game/state.rs`: `PuzzleState` struct — the full state to serialize (grid, user_grid, clues, selection, difficulty)
- `src/game/history.rs`: `WordHistory` — currently in-memory VecDeque, needs replacement with `user.db`-backed decay model
- `src/input/handler.rs`: `InputState.clue_ratings: HashMap<i64, bool>` — currently in-memory, needs `user.db` backing
- `src/db/mod.rs`: `open_database()`, query layer — pattern to follow for `user.db`
- `src/db/schema.rs`: `init_schema()` — pattern for `user.db` schema initialization

### Established Patterns
- `GameState` enum with `DifficultySelection` / `Generating` / `Playing` / `Congratulations` — menu flow changes add states or modify transitions
- Background thread generation via `mpsc::channel` — puzzle generation already async, save/load hooks into existing flow
- `PuzzleState::from_filled_grid()` runs in background thread with `Connection` — `user.db` writes need same threading consideration (Connection is Send not Sync)

### Integration Points
- `src/main.rs` game loop — save triggers after `process_input()`, load on menu resume click
- `src/render/menu.rs` — needs "Onvoltooide puzzels" button and puzzle list UI
- `src/render/menu.rs` — difficulty selection screen needs "Terug naar puzzel" button
- `src/db/mod.rs` — `words_for_length()` query needs `ATTACH DATABASE` join for thumbs-down filtering from `user.db`
- `src/grid/generator.rs` — exclude set construction changes from in-memory history to `user.db` decay query

</code_context>

<deferred>
## Deferred Ideas

- LLM-driven clue quality analysis from accumulated feedback data — v2
- Puzzle difficulty rating (1-10 post-completion prompt, D-05 from Phase 1) — v2
- Cloud save / online sync — explicitly out of scope

</deferred>

---

*Phase: 03-session-continuity*
*Context gathered: 2026-03-22*
