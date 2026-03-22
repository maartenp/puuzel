---
phase: 02-playable-game
plan: 04
subsystem: generator
tags: [csp, word-history, hashset, exclude, backtracking]

# Dependency graph
requires:
  - phase: 02-playable-game
    provides: "WordHistory struct with recent_ids() (02-01), generate_grid CSP with used_ids filtering (01-03)"
provides:
  - "generate_grid accepts exclude HashSet parameter for word history filtering"
  - "PGEN-04 gap closed: word history flows end-to-end from main.rs to CSP used_ids"
affects: [puzzle-generation, word-variety]

# Tech tracking
tech-stack:
  added: []
  patterns: ["exclude set seeded into CSP used_ids for cross-puzzle word deduplication"]

key-files:
  created: []
  modified:
    - src/grid/generator.rs
    - src/main.rs

key-decisions:
  - "Exclude set integrated at CSP used_ids level (not WordIndex::build) -- minimal change, leverages existing candidates_for_constraints filtering"

patterns-established:
  - "generate_grid exclude parameter: all callers must pass &HashSet<i64>, tests use &HashSet::new()"

requirements-completed: [PGEN-04]

# Metrics
duration: 2min
completed: 2026-03-22
---

# Phase 02 Plan 04: Word History Wiring Summary

**Wire word history exclude set into CSP generator via generate_grid parameter, closing PGEN-04 gap**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-22T09:37:38Z
- **Completed:** 2026-03-22T09:39:17Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- generate_grid now accepts `exclude: &HashSet<i64>` parameter for word history filtering
- CSP backtracker seeds used_ids from exclude set instead of empty HashSet
- Dead code (`let _ = exclude`) removed from main.rs; exclude set passed through background thread
- All 7 existing generator tests pass with empty exclude sets (no behavioral change)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add exclude parameter to generate_grid and seed CSP used_ids** - `1ad1f03` (feat)
2. **Task 2: Wire exclude set from main.rs into background thread and generate_grid call** - `ca33ac7` (feat)

## Files Created/Modified
- `src/grid/generator.rs` - Added exclude parameter to generate_grid, seed used_ids from exclude.clone(), updated 9 test call sites
- `src/main.rs` - Removed dead `let _ = exclude`, pass exclude into move closure and generate_grid call

## Decisions Made
- Integrated exclude at CSP used_ids level rather than WordIndex::build -- this is the minimal correct integration point since candidates_for_constraints already filters by used_ids

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- PGEN-04 fully wired: WordHistory.recent_ids() -> HashSet -> background thread -> generate_grid -> CSP used_ids -> candidates_for_constraints filtering
- Word variety across consecutive puzzles is now enforced (last 200 words excluded)

## Self-Check: PASSED

- Files: src/grid/generator.rs FOUND, src/main.rs FOUND
- Commits: 1ad1f03 FOUND, ca33ac7 FOUND
- exclude parameter in generate_grid signature: confirmed
- exclude.clone() seeds used_ids: confirmed
- Dead code removed from main.rs: confirmed
- generate_grid called with &exclude in main.rs: confirmed

---
*Phase: 02-playable-game*
*Completed: 2026-03-22*
