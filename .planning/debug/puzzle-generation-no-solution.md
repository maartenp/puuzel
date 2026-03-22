---
status: verifying
trigger: "puzzle-generation-no-solution"
created: 2026-03-22T00:00:00Z
updated: 2026-03-22T00:03:00Z
---

## Current Focus

hypothesis: CONFIRMED. Three root causes found via diagnostic logging and fixed.
test: cargo test (57 pass), diagnostic test (all 3 difficulties succeed on first attempt)
expecting: All difficulties generate puzzles successfully in the running app
next_action: Await human verification in the app

## Symptoms

expected: Puzzles generate successfully for all three difficulty levels (easy, medium, hard)
actual: Generation fails with "Generation failed: No solution found for the given constraints" for easy, medium, and hard puzzles despite 35,107 clues in the database (4,974 easy, 11,273 medium, 18,860 hard)
errors: "Generation failed: No solution found for the given constraints"
reproduction: Select any difficulty level and attempt to generate a puzzle
started: Database has clues but puzzles fail to generate

## Eliminated

- hypothesis: General database connectivity / query issues
  evidence: DB has 35,107 clues, queries in db/mod.rs look correct
  timestamp: 2026-03-22

- hypothesis: WordIndex loads wrong length range (previous fix attempt)
  evidence: Loading 2..=max_word_length still failed — Easy grids always have slots >8, CSP fails
  timestamp: 2026-03-22

- hypothesis: min_commonness too high (previous fix attempt)
  evidence: Lowering min_commonness from 4 to 2 expanded word pool but didn't fix the bottleneck
  timestamp: 2026-03-22

## Evidence

- timestamp: 2026-03-22
  checked: src/grid/generator.rs WordIndex::build()
  found: Loop `for length in 2..=20` loads words for ALL lengths regardless of config.max_word_length. The `viable` check at line 224 only verifies words exist for each slot length — but slots exceeding max_word_length will have words in the index (from the DB) yet violate the design intent of restricting Easy to 8-char max.
  implication: Easy grids can have slots up to length 20 (full row/column of white cells), and the viable check passes because Hard-length words exist. But the easy-filtered pool doesn't include those words, causing CSP failure on those slots.

- timestamp: 2026-03-22
  checked: src/grid/types.rs DifficultyConfig::easy()
  found: min_commonness=4, max_word_length=8. Medium has min_commonness=3, Hard has min_commonness=1.
  implication: Easy pool is severely restricted. With only ~990 words at commonness>=4 and length<=8, backtracking depletes candidates quickly under crossing constraints.

- timestamp: 2026-03-22
  checked: src/grid/difficulty.rs extract_slots() and seed_black_squares()
  found: seed_black_squares produces grids with black ratio 35-40% for Easy (more black = shorter average slots). extract_slots() collects all white runs of length>=2 with no upper bound. On a 20x20 grid with 35% black squares, many slots will exceed length 8.
  implication: Slots of length 9-20 are common even in Easy grids. With Easy's pool restricted to length<=8, those slots have zero candidates => CSP fails immediately on all 5 shape attempts.

- timestamp: 2026-03-22 (second investigation round via diagnostic logging)
  checked: Added eprintln! diagnostics to generate_grid_inner; ran test_real_db_diagnostic against real DB
  found: (1) Easy: all 20/50 shape attempts produce slots >8 (length 9-18 common). Root cause: you CANNOT reliably constrain slot length through black square placement. (2) CRITICAL: DB has only 8 2-letter words for Hard, 6 for Medium. Grids had 30-60 two-letter slots. CSP fails immediately on 2-letter slot bottleneck. (3) words_for_length() filters by difficulty='hard' etc, but short words (2-3 letter) mostly only have 'easy' clues — so Hard/Medium word pools for short words are tiny.
  implication: Previous fixes were insufficient. Need: (a) use any-clue word pool for grid placement (decouple clue difficulty from word placement), (b) enforce min_word_length=3 in grid shape to eliminate 2-letter slots, (c) load words for all lengths up to 20 (abandon max_word_length enforcement in generation).

## Resolution

root_cause: Three compounding root causes: (1) Grid generation always produces 2-letter slots (30-60 per grid), but only 6-8 two-letter words exist for Hard/Medium difficulty levels. CSP fails immediately on 2-letter slot bottleneck. (2) words_for_length() filters by clue difficulty='hard'/'medium', but short words (2-3 letter) mostly only have 'easy' difficulty clues — dramatically reducing the word pool for Hard/Medium. (3) max_word_length enforcement was impossible: random black square placement can't reliably keep all slots ≤8 cells, making Easy grids nearly always invalid (all 50 attempts rejected).
fix: (1) Added words_for_length_any_clue() in db/mod.rs — selects words with ANY clue at any difficulty, filtered only by commonness_score. Grid placement now uses this function (word pool decoupled from clue display difficulty). (2) Changed min_word_length from 2 to 3 in all DifficultyConfigs. Updated extract_slots() to accept min_length parameter. Added fix_short_slots() to replace fix_length_one_slots() — eliminates cells in sub-minimum runs in BOTH directions. Added orphan cell check in generator to reject shapes with uncovered white cells. (3) WordIndex::build() now loads lengths min_word_length..=20 (not limited to max_word_length). Clue fallback added in from_filled_grid: tries requested difficulty first, then any other difficulty. Shape/seed attempt count increased from 20 to 50.
verification: cargo test: 57 passed, 0 failed. test_real_db_diagnostic: all 3 difficulties succeed on first shape attempt, total <5 seconds.
files_changed: [src/grid/generator.rs, src/grid/difficulty.rs, src/grid/types.rs, src/db/mod.rs, src/game/state.rs]
