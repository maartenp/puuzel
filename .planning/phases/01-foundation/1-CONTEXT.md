# Phase 1: Foundation - Context

**Gathered:** 2026-03-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Grid engine that generates valid Dutch/European-style crossword grids (~20x20) and a verified Dutch word+clue database bundled with the app in SQLite format. This phase delivers the data layer and generation logic — no rendering, no UI, no user interaction.

</domain>

<decisions>
## Implementation Decisions

### Difficulty calibration
- **D-01:** All difficulties use 20x20 grids — no size variation
- **D-02:** Black square ratio: easy ~35-40%, medium ~30-35%, hard ~25-30% (European newspaper range)
- **D-03:** LLM rates each clue with a difficulty level during generation; difficulties pick from their rated pool
- **D-04:** LLM also rates word commonness — easy puzzles avoid obscure words entirely
- **D-05:** Difficulty calibration is iterative — v1 ships with LLM-rated defaults, users rate completed puzzles 1-10 (prompted after completion) to guide future tuning
- **D-06:** Difficulty rating is internal only — never shown to the user

### Word selection criteria
- **D-07:** No abbreviations in the word list
- **D-08:** Proper nouns allowed — place names (Amsterdam, Rijn) and people (Rembrandt, Cruijff) are fair game
- **D-09:** Maximum word length: 15 characters
- **D-10:** Minimum word length: 2 characters (European convention permits two-letter words)
- **D-11:** Auto-filter vulgar/offensive words from the word list
- **D-12:** Archaic/literary Dutch words are included if the clue explicitly mentions they are archaic (e.g., "Ouderwets woord voor 'graag'")
- **D-13:** Compound words welcome up to the 15-char cap

### Clue style per difficulty
- **D-14:** Easy clues: direct synonyms or conversational definitions (both styles acceptable)
- **D-15:** Hard clues: indirect/lateral definitions requiring inference or specific knowledge
- **D-16:** Medium clues: factual but requiring more thought than easy — sits between direct and lateral
- **D-17:** Maximum clue length: 6 words
- **D-18:** All clues are straightforward definitions — no cryptic or wordplay clues
- **D-19:** Clues generated in Dutch only for v1

### Clue feedback system
- **D-20:** Double-click a clue to rate it: shows easy/medium/hard difficulty rating AND thumbs up/thumbs down
- **D-21:** Thumbs down means "never show this clue again" — hard blacklist
- **D-22:** Feedback data persisted for future LLM-driven pattern analysis (v2)
- **D-23:** Word-level thumbs up/down also available to flag bad words

### IJ digraph handling
- **D-24:** IJ occupies a single cell in the grid — "IJSBEER" = 6 cells (IJ-S-B-E-E-R)
- **D-25:** Cell displays "IJ" (two ASCII characters), not the Unicode ligature
- **D-26:** Input: user presses I then J (no other key in between) → merges into single IJ cell
- **D-27:** No time limit on the I→J sequence — purely sequential keypress detection
- **D-28:** Backspace on an IJ cell: first press reverts to "I" in the cell, second press clears it
- **D-29:** Words in the database store IJ as a single token for grid-fitting purposes (counts as 1 letter toward grid placement)

### Word list source
- **D-30:** Use the pre-downloaded `dutch.txt` file in the repo root (~190K words) — no need to download from OpenTaal at runtime
- **D-31:** The file still needs the same filtering (length, abbreviations, vulgarity, etc.)

### Clue generation method
- **D-32:** Use Claude Code CLI (`claude -p`) via subprocess instead of Claude Batch API — leverages Max subscription, no API key needed
- **D-33:** Process words in batches of 10K with a bash runner script
- **D-34:** If usage limit is hit, wait for reset and automatically resume
- **D-35:** Script can be interrupted and resumed — picks up from last completed batch

### Claude's Discretion
- Grid generation algorithm choice (constraint propagation, backtracking strategy)
- OpenTaal word list filtering pipeline (inflection removal, frequency scoring methodology)
- Claude Code CLI prompt design for clue generation and self-verification
- SQLite schema design for word/clue storage
- Word frequency/commonness scoring approach
- IJ detection in the dutch.txt word list (normalization strategy)

</decisions>

<specifics>
## Specific Ideas

- Difficulty should feel like Dutch newspaper crossword difficulty — easy is "Telegraaf", hard is "NRC Handelsblad"
- Archaic words add flavor to hard puzzles but must be clued honestly ("Ouderwets woord voor...")
- The 1-10 puzzle rating is a lightweight post-completion prompt, not a complex feedback form
- Thumbs down on a clue is a strong signal — that clue is permanently retired

</specifics>

<canonical_refs>
## Canonical References

### Grid engine
- `.planning/REQUIREMENTS.md` §Grid Engine — GRID-01 through GRID-08: grid validity rules, IJ handling, difficulty-dependent density/length/commonness
- `CLAUDE.md` §Technology Stack — macroquad, rand, rusqlite versions and usage patterns

### Word/clue database
- `.planning/REQUIREMENTS.md` §Word & Clue Database — DATA-01 through DATA-06: word sourcing, clue generation, self-verification, SQLite bundling
- `CLAUDE.md` §Word / Clue Databases — OpenTaal source, SCOWL for English (v2), bundled database format, rusqlite with bundled feature

### Project constraints
- `.planning/PROJECT.md` §Constraints — No runtime API calls, bundled data, Rust+macroquad stack
- `.planning/PROJECT.md` §Context — Dutch crossword conventions, target user profile

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- No existing code — greenfield project

### Established Patterns
- No patterns yet — this phase establishes the foundational patterns

### Integration Points
- Grid engine output consumed by Phase 2 (rendering + puzzle generation)
- SQLite database consumed by Phase 2 (word selection during puzzle generation)
- IJ handling conventions established here must be consistent through all future phases

</code_context>

<deferred>
## Deferred Ideas

- LLM-driven clue quality analysis from user feedback data — v2 feature
- English word+clue database (SCOWL + AI clues) — v2 (LANG-01 through LANG-04)
- Bilingual clue switching — v2
- Puzzle rating 1-10 data analysis and difficulty rebalancing — v2
- Clue feedback pattern detection via LLM — v2

</deferred>

---

*Phase: 01-foundation*
*Context gathered: 2026-03-21*
