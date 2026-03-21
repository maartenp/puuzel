---
phase: 01-foundation
verified: 2026-03-21T22:00:00Z
status: gaps_found
score: 3/5 success criteria verified
re_verification: false
gaps:
  - truth: "The bundled SQLite database contains Dutch words with clues at easy, medium, and hard difficulty, each clue having passed an AI self-verification round"
    status: failed
    reason: "All 478 clues in data/puuzel.db have verified=0. The self-verification pipeline ran but produced 0 verified clues. The words_for_length Rust function was silently changed to drop the verified=1 filter (now accepts any non-thumbs-down clue) to work around this, but this means DATA-06 is not satisfied."
    artifacts:
      - path: "data/puuzel.db"
        issue: "478 clues, 0 with verified=1 (SELECT COUNT(*) FROM clues WHERE verified=1 returns 0)"
      - path: "src/db/mod.rs"
        issue: "words_for_length query omits AND c.verified=1 — plan required verified-only clues; implementation accepts any thumbs_down=0 clue"
    missing:
      - "Re-run self-verification step or debug why verify_clues_chunk returns 0 verified clues"
      - "Restore verified=1 filter in words_for_length OR document the intentional relaxation"
  - truth: "AI-generated clues for each word at three difficulty levels (easy, medium, hard)"
    status: failed
    reason: "Implementation generates ONE clue per word and assigns a single difficulty level derived from commonness score. The plan spec required three separate clues (easy/medium/hard) per word. The DB has no word with clues at multiple difficulty levels."
    artifacts:
      - path: "tools/generate_clues.py"
        issue: "generate_clues_for_chunk prompt asks for one clue per word, not three. commonness_to_difficulty() maps commonness to a single difficulty bucket."
      - path: "data/puuzel.db"
        issue: "No word in the DB has clues at more than one difficulty level. Distribution: easy=68, medium=132, hard=278 across all words."
    missing:
      - "Either update generate_clues_for_chunk to generate 3 clues per word (easy/medium/hard) or document the design decision that one-clue-per-word with commonness-derived difficulty is the intended approach"
  - truth: "Archaic/literary word hard clues prefixed with 'Ouderwets woord voor'"
    status: failed
    reason: "The generate_clues.py prompt detects and stores is_archaic, but does NOT inject the 'Ouderwets woord voor' prefix into hard clues for archaic words. This was requirement D-12 from the context."
    artifacts:
      - path: "tools/generate_clues.py"
        issue: "Prompt does not contain 'Ouderwets woord voor'. is_archaic is captured but the prefix is never applied to clues."
    missing:
      - "Add 'Als het woord is_archaic=true is, begin de aanwijzing met Ouderwets woord voor' to the clue generation prompt"
  - truth: "Database is sufficient for crossword generation (adequate word coverage across all grid lengths)"
    status: failed
    reason: "DB contains only 500 words. No 2-letter words exist in the database (grid_length=2 has 0 entries). The word pool is too small for reliable grid generation — the generator test suite uses an in-memory DB, not puuzel.db. Only 500 of 164K filtered words have been processed."
    artifacts:
      - path: "data/puuzel.db"
        issue: "Only 500 words (0 with grid_length=2, 4 with grid_length=3, 7 with grid_length=4). The full 164K filtered word list has not been processed through clue generation."
    missing:
      - "Run the full clue generation pipeline on the complete 164K filtered word list"
      - "Ensure 2-letter words (IJS, OP, IN, etc.) are included in the pipeline output"
---

# Phase 1: Foundation Verification Report

**Phase Goal:** A valid Dutch/European-style crossword grid can be generated and a verified Dutch word+clue database exists bundled with the app
**Verified:** 2026-03-21T22:00:00Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | App generates 20x20 Dutch/European-style grid with connected white squares, completing in under 10 seconds | ✓ VERIFIED | generate_grid exists, seeding + CSP + 8s timeout, 43 tests pass |
| 2 | IJ digraph occupies a single cell in any grid containing a Dutch word with IJ | ✓ VERIFIED | LetterToken::IJ in types.rs, tokenize_dutch_word handles all cases, 10 IJ tests pass |
| 3 | Bundled SQLite database contains Dutch words with clues at easy/medium/hard difficulty, each having passed AI self-verification | ✗ FAILED | data/puuzel.db has 478 clues, ALL with verified=0. Zero self-verified clues. |
| 4 | Generator respects unchecked letters and permits two-letter words, matching European grid conventions | ✓ VERIFIED | extract_slots accepts length>=2, no enforcement of "checked" constraint, tests confirm |
| 5 | Grid black-square density and word length distribution visibly differ between easy and hard difficulty | ✓ VERIFIED | DifficultyConfig::easy() ratio 0.35-0.40, hard() 0.25-0.30; max_word_length 8 vs 15; density tests pass |

**Score:** 3/5 success criteria verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `Cargo.toml` | Project manifest with Phase 1 deps | ✓ VERIFIED | macroquad 0.4, rusqlite 0.39 bundled, rand 0.10, serde, log all present |
| `src/grid/types.rs` | Core grid types: Cell, Direction, Slot, DifficultyConfig, LetterToken | ✓ VERIFIED | All types present and exported, 4 tests |
| `src/grid/ij.rs` | IJ digraph tokenization | ✓ VERIFIED | tokenize_dutch_word, grid_length, 10 tests covering all edge cases |
| `src/grid/connectivity.rs` | Flood-fill connectivity check | ✓ VERIFIED | is_connected BFS, 6 tests |
| `src/grid/difficulty.rs` | Black square seeding + slot extraction | ✓ VERIFIED | seed_black_squares, extract_slots, 9 tests |
| `src/grid/generator.rs` | CSP backtracking generator | ✓ VERIFIED | generate_grid, WordIndex, FilledGrid, GeneratorError, 7 tests (1 ignored) |
| `src/db/schema.rs` | SQLite CREATE TABLE statements | ✓ VERIFIED | words + clues tables, 4 indexes via init_schema |
| `src/db/mod.rs` | Database open and query functions | ✓ VERIFIED | open_database, open_in_memory, insert_word, insert_clue, words_for_length, get_clue_for_word — 6 tests. NOTE: words_for_length deviates from plan: no verified=1 filter |
| `tools/filter_wordlist.py` | Dutch word filter pipeline | ✓ VERIFIED | compute_grid_length, filter_word, normalize_word, reads local dutch.txt, 13 tests pass |
| `tools/dutch_blocklist.txt` | Dutch vulgarity blocklist | ✓ VERIFIED | 108 lines |
| `tools/generate_clues.py` | Claude Code CLI clue generation | ✓ VERIFIED (structurally) | generate_clues_for_chunk, verify_clues_chunk, RateLimitError, CHUNK_SIZE=50, claude-haiku model. DEVIATION: generates 1 clue/word not 3 |
| `tools/generate_clues_batch.sh` | Batch runner | ✓ VERIFIED | BATCH_SIZE, sleep 3600, clues_batch_ files, verified_clues.json |
| `tools/write_database.py` | Database writer | ✓ VERIFIED | CREATE TABLE words, CREATE TABLE clues, puuzel.db, is_archaic handling |
| `data/puuzel.db` | Bundled SQLite database with verified clues | ✗ STUB | Exists, has 500 words, but 0 verified clues. No 2-letter words. Insufficient coverage. |
| `tools/output/filtered_words.json` | Filtered Dutch words | ✓ VERIFIED | 164,730 words filtered from dutch.txt |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/grid/ij.rs` | `src/grid/types.rs` | uses LetterToken enum | ✓ WIRED | `use crate::grid::types::LetterToken;` at line 1 |
| `src/db/mod.rs` | `src/db/schema.rs` | calls init_schema on DB open | ✓ WIRED | `schema::init_schema(&conn)?` in both open_database and open_in_memory |
| `src/grid/generator.rs` | `src/grid/types.rs` | uses Grid, Cell, Slot, etc. | ✓ WIRED | `use crate::grid::types::{Cell, Difficulty, DifficultyConfig, Direction, Grid, LetterToken, Slot};` |
| `src/grid/generator.rs` | `src/grid/connectivity.rs` | calls is_connected | ✓ WIRED | `use crate::grid::connectivity::is_connected;` — called during seeding and final validation |
| `src/grid/generator.rs` | `src/grid/difficulty.rs` | calls seed_black_squares | ✓ WIRED | `use crate::grid::difficulty::{extract_slots, seed_black_squares};` |
| `src/grid/generator.rs` | `src/db/mod.rs` | calls words_for_length | ✓ WIRED | `use crate::db;` then `db::words_for_length(conn, ...)` in WordIndex::build |
| `tools/filter_wordlist.py` | `tools/dutch_blocklist.txt` | reads blocklist | ✓ WIRED | `dutch_blocklist.txt` referenced, loaded as set |
| `tools/filter_wordlist.py` | `dutch.txt` | reads Dutch word list | ✓ WIRED | reads from repo root |
| `tools/generate_clues_batch.sh` | `tools/generate_clues.py` | calls Python script in batches | ✓ WIRED | `python tools/generate_clues.py` in batch loop |
| `tools/write_database.py` | verified_clues.json output | reads JSON to write SQLite | ✓ WIRED | reads verified_clues.json, inserts into puuzel.db |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| GRID-01 | 01-02 | App generates valid Dutch/European-style crossword grids (~20x20) | ✓ SATISFIED | generate_grid produces 20x20 Grid; 43 tests pass |
| GRID-02 | 01-02 | Generated grids have connected white squares | ✓ SATISFIED | is_connected BFS enforced during seeding and post-generation; tests verify |
| GRID-03 | 01-01 | IJ digraph treated as single cell | ✓ SATISFIED | LetterToken::IJ, tokenize_dutch_word, grid_length all correct |
| GRID-04 | 01-02 | Unchecked letters permitted | ✓ SATISFIED | No enforcement that letters must be checked; crossing map only constrains, does not require |
| GRID-05 | 01-02 | Two-letter words permitted | ✓ SATISFIED | extract_slots includes length=2 slots; test_two_letter_slots_exist verifies |
| GRID-06 | 01-02 | Black square density varies by difficulty | ✓ SATISFIED | easy 0.35-0.40, hard 0.25-0.30; density tests pass |
| GRID-07 | 01-02 | Word length varies by difficulty | ✓ SATISFIED | DifficultyConfig: easy max_word_length=8, hard max_word_length=15 |
| GRID-08 | 01-02 | Word commonness varies by difficulty | ✓ SATISFIED | DifficultyConfig: easy min_commonness=4, hard min_commonness=1 |
| DATA-01 | 01-03 | Dutch word list sourced and filtered | ✓ SATISFIED | 164,730 words in filtered_words.json from dutch.txt |
| DATA-02 | 01-03 | AI-generated clues at three difficulty levels | ✗ BLOCKED | Implementation generates 1 clue/word (difficulty from commonness). No word has clues at all three levels. |
| DATA-03 | 01-03 | Clues are straightforward definitions | ? NEEDS HUMAN | Prompt enforces "no woordspeling" but output not manually reviewed at scale |
| DATA-04 | 01-01 | Word+clue database bundled in SQLite format | ✓ SATISFIED | data/puuzel.db exists with correct schema |
| DATA-05 | 01-01 | Database includes word frequency/commonness metadata | ✓ SATISFIED | commonness_score column in words table, populated from LLM |
| DATA-06 | 01-03 | AI clue generation includes self-verification pass | ✗ BLOCKED | verify_clues_chunk exists and runs but produced 0 verified clues. All 478 DB clues have verified=0. |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/db/mod.rs` | 63 | words_for_length omits `AND c.verified = 1` filter — plan required verified-only clues | ⚠️ Warning | Works around 0-verified-clue problem but silently changes contract. With 0 verified clues in DB, adding verified=1 would make generator fail to find any words. |
| `data/puuzel.db` | — | 500 words, 0 verified clues, no 2-letter words | ✗ Blocker | Database insufficient for production use; grid generation with real DB would fail |
| `tools/generate_clues.py` | 58-70 | Prompt generates 1 clue/word, not 3 difficulty levels | ✗ Blocker | DATA-02 requires easy/medium/hard clues per word; current approach generates one |

### Human Verification Required

#### 1. Clue Quality Check

**Test:** Open `tools/output/quality_sample.json` and review the 10 sample clues. Check that they are appropriate Dutch crossword clues (not too long, no wordplay, descriptive).
**Expected:** Each clue is a clear Dutch synonym or short definition, max 6 words, no part of the target word appears in the clue.
**Why human:** Semantic quality of language cannot be verified programmatically.

#### 2. Self-Verification Pipeline Debugging

**Test:** Run `python tools/generate_clues.py --start 0 --count 10` with verbose output and observe whether the verification step prints any verified=True results.
**Expected:** Some clues should pass verification (model answers its own clue with the correct word).
**Why human:** Requires running the claude CLI interactively to see where verification fails. Likely the issue is that Haiku's answer to the verification prompt doesn't exactly match the uppercase target word.

### Gaps Summary

Phase 1 has a complete and well-tested Rust grid engine (all GRID-* requirements satisfied, 43 tests passing) but the data pipeline has critical gaps:

**DATA-06 (self-verification) is broken:** The `verify_clues_chunk` function exists and the pipeline ran, but produced 0 verified clues out of 478 generated. The exact failure mode is unknown without running the CLI interactively — likely Haiku's verification answers don't match the target words (case mismatch, inflected forms, or the verification JSON parse is silently failing). The Rust `words_for_length` query was silently modified to work around this (drops the `verified=1` filter), masking the problem.

**DATA-02 (three difficulty levels per word) was redesigned:** The implementation generates one clue per word and uses commonness score to assign a single difficulty bucket. This is a significant deviation from the plan spec. Words do not have separate easy/medium/hard clue variants. Whether this is intentional or an oversight needs clarification.

**Database is a pilot run only:** 500 words were processed (vs 164K available). The database lacks 2-letter words entirely (critical for GRID-05 in production). The generator tests use an in-memory database with hardcoded words, not `data/puuzel.db`, so generator tests pass regardless of DB state.

The phase's Rust foundation (grid engine + schema layer) is solid. The data pipeline tooling is structurally complete but needs the verification bug fixed and the full pipeline executed before Phase 1 can be considered done.

---

_Verified: 2026-03-21T22:00:00Z_
_Verifier: Claude (gsd-verifier)_
