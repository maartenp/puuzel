---
phase: 01-foundation
plan: 03
subsystem: data-pipeline
tags: [dutch, wordlist, clue-generation, sqlite, python, tdd]
dependency_graph:
  requires: [dutch.txt in repo root]
  provides: [tools/filter_wordlist.py, tools/generate_clues.py, tools/generate_clues_batch.sh, tools/write_database.py, data/puuzel.db (after user runs pipeline)]
  affects: [Phase 2 word selection — all runtime word queries depend on data/puuzel.db]
tech_stack:
  added: [Python 3, pytest, sqlite3 (stdlib), subprocess (claude CLI via Max subscription)]
  patterns: [IJ-aware token counting, TDD filter pipeline, incremental batch processing with auto-retry]
key_files:
  created:
    - tools/filter_wordlist.py
    - tools/dutch_blocklist.txt
    - tools/tests/test_filter.py
    - tools/tests/__init__.py
    - tools/requirements.txt
    - tools/generate_clues.py
    - tools/generate_clues_batch.sh
    - tools/write_database.py
    - tools/output/filtered_words.json
  modified: []
decisions:
  - "IJ digraph handled as 2-char token 'IJ' not Unicode ligature U+0132 — consistent with D-24/D-25"
  - "Blocklist approach for vulgarity filtering: ~100-word best-effort list for v1, easily extensible"
  - "Only verified clues (LLM self-verification pass) are inserted into SQLite — DATA-06"
  - "claude-haiku-4-5-20251001 model for clue generation via Max subscription CLI (D-32)"
metrics:
  duration_seconds: 208
  completed_date: "2026-03-21"
  tasks_completed: 2
  tasks_total: 3
  files_created: 9
---

# Phase 01 Plan 03: Dutch Word Filter and Clue Generation Pipeline Summary

**One-liner:** Offline Dutch crossword data pipeline — IJ-aware word filter + claude CLI clue generator + SQLite writer with self-verification pass.

## What Was Built

### Task 1: Word list filter pipeline (TDD)

The filter reads `dutch.txt` (189,904 words) from the repo root and produces `tools/output/filtered_words.json` (164,730 words after filtering).

Key behaviors:
- `compute_grid_length("IJSBEER") == 6` — IJ counts as one cell (D-24)
- Unicode IJ ligature U+0132 normalized before counting
- Words with dots (abbreviations), digits, hyphens excluded (D-07, D-13)
- Vulgarity blocklist of ~100 Dutch offensive words applied (D-11)
- Grid length bounds: 2–15 cells (D-09, D-10)
- Proper noun detection: starts uppercase, not all-caps

13 pytest tests written first (TDD RED), then implementation (GREEN). All pass.

### Task 2: Clue generator + batch runner + database writer

**tools/generate_clues.py:**
- Batches 50 words per `claude -p` call using `claude-haiku-4-5-20251001` via Max subscription
- Dutch clue prompt includes archaic word rule: "Ouderwets woord voor" prefix for hard clues (D-12)
- `is_archaic` boolean returned by LLM and stored
- Self-verification pass: asks model to answer each clue, marks `verified: true` only on exact word match (DATA-06)
- Quality gate after first 50-word chunk: prints 10 sample words, asks "Continue? [y/n]"
- `--no-gate` flag for unattended subsequent batches
- RateLimitError on rate limit, saves progress and exits with code 2
- Incremental saves every 500 words

**tools/generate_clues_batch.sh:**
- Processes full word list in batches of 10K (configurable via `BATCH_SIZE` env)
- Detects rate limit (exit code 2), waits 60 minutes with `sleep 3600`, retries
- Resumes from last completed batch file
- Merges all batch files to `verified_clues.json` on completion

**tools/write_database.py:**
- Creates `data/puuzel.db` with schema matching Rust `db/schema.rs`
- Tables: `words` (id, word, grid_length, commonness_score, is_proper_noun, is_archaic), `clues` (id, word_id, difficulty, clue_text, verified, thumbs_down)
- 4 indexes: grid_length, commonness, word+difficulty, verified
- Only inserts verified clues
- Prints validation queries: word count, clues per difficulty, length distribution, archaic count

### Task 3: Run the pipeline (CHECKPOINT — awaiting human action)

Task 3 requires the user to run the pipeline using their Claude Max subscription. See checkpoint details below.

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

- `tools/output/filtered_words.json` exists (164,730 words) — ready for clue generation
- `data/puuzel.db` does NOT yet exist — will be created when user runs the pipeline (Task 3)

## Self-Check: PASSED

- tools/filter_wordlist.py: FOUND
- tools/dutch_blocklist.txt: FOUND
- tools/tests/test_filter.py: FOUND
- tools/generate_clues.py: FOUND
- tools/generate_clues_batch.sh: FOUND
- tools/write_database.py: FOUND
- tools/output/filtered_words.json: FOUND
- All pytest tests: 13 passed
- generate_clues imports: ok
- write_database imports: ok
- batch script executable: ok
