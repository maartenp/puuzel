---
phase: 1
slug: foundation
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-03-21
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + pytest (Python pipeline) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test -- --include-ignored` |
| **Estimated runtime** | ~15 seconds (Rust), ~5 seconds (Python) |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test -- --include-ignored`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01 Task 1 | 01-01 | 1 | GRID-03 | unit | `cargo test grid -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-01 Task 2 | 01-01 | 1 | DATA-04, DATA-05 | unit | `cargo test db -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-02 Task 1 | 01-02 | 2 | GRID-02, GRID-06 | unit | `cargo test grid::connectivity::tests -- --nocapture && cargo test grid::difficulty::tests -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-02 Task 2 | 01-02 | 2 | GRID-01, GRID-04, GRID-05, GRID-07, GRID-08 | unit | `cargo test grid::generator::tests -- --nocapture` | ❌ W0 | ⬜ pending |
| 01-03 Task 1 | 01-03 | 1 | DATA-01 | unit | `python -m pytest tools/tests/test_filter.py -v` | ❌ W0 | ⬜ pending |
| 01-03 Task 2 | 01-03 | 1 | DATA-02, DATA-03, DATA-06 | integration | `python -c "from tools.generate_clues import make_generation_request, make_verification_request; print('imports ok')" && python -c "from tools.write_database import create_database; print('imports ok')"` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Requirement Coverage

| Requirement | Covered By | Automated Command |
|-------------|------------|-------------------|
| GRID-01 | 01-02 Task 2 | `cargo test grid::generator::tests` |
| GRID-02 | 01-02 Task 1 | `cargo test grid::connectivity::tests` |
| GRID-03 | 01-01 Task 1 | `cargo test grid` |
| GRID-04 | 01-02 Task 2 | `cargo test grid::generator::tests` |
| GRID-05 | 01-02 Task 2 | `cargo test grid::generator::tests` |
| GRID-06 | 01-02 Task 1 + Task 2 | `cargo test grid::difficulty::tests && cargo test grid::generator::tests` |
| GRID-07 | 01-02 Task 2 | `cargo test grid::generator::tests` |
| GRID-08 | 01-02 Task 2 | `cargo test grid::generator::tests` |
| DATA-01 | 01-03 Task 1 | `python -m pytest tools/tests/test_filter.py` |
| DATA-02 | 01-03 Task 2 | import smoke test |
| DATA-03 | 01-03 Task 2 | import smoke test |
| DATA-04 | 01-01 Task 2 | `cargo test db` |
| DATA-05 | 01-01 Task 2 | `cargo test db` |
| DATA-06 | 01-03 Task 2 | import smoke test |

---

## Wave 0 Requirements

- [ ] `Cargo.toml` — project scaffold with dependencies (macroquad, rusqlite, rand, serde, serde_json, log, env_logger)
- [ ] `src/main.rs` — minimal entry point
- [ ] `src/grid/mod.rs` — module declarations
- [ ] `src/db/mod.rs` + `src/db/schema.rs` — database module stubs
- [ ] `tools/tests/__init__.py` — empty file for pytest discovery

*Wave 0 tasks are the first task of Plan 01-01 (Cargo scaffold + types) and the first task of Plan 01-03 (filter + test infra). These establish the test infrastructure all other tasks verify against.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Grid visual inspection | GRID-01 | Grid structure needs human eye to confirm it "looks like" a Dutch crossword | Print grid to stdout, compare against newspaper reference |
| Clue quality spot-check | DATA-03 | Clue "straightforwardness" is subjective | Review 20 random clues at each difficulty level |
| Archaic clue prefix check | D-12 | Confirm hard clues for archaic words actually start with "Ouderwets woord voor" | Inspect `SELECT w.word, c.clue_text FROM words w JOIN clues c ON w.id=c.word_id WHERE w.is_archaic=1 AND c.difficulty='hard' LIMIT 20` in data/puuzel.db |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [ ] Wave 0 complete (tasks not yet executed)
- [x] No watch-mode flags
- [x] Feedback latency < 15s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
