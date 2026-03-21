---
phase: 1
slug: foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test -- --include-ignored` |
| **Estimated runtime** | ~15 seconds |

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
| TBD | TBD | TBD | GRID-01 | unit | `cargo test grid` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-02 | unit | `cargo test grid::connectivity` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-03 | unit | `cargo test ij` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-04 | unit | `cargo test grid::unchecked` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-05 | unit | `cargo test grid::two_letter` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-06 | unit | `cargo test grid::density` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-07 | unit | `cargo test grid::word_length` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | GRID-08 | unit | `cargo test grid::commonness` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-01 | integration | `cargo test data::wordlist` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-02 | integration | `cargo test data::clues` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-03 | unit | `cargo test data::clue_style` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-04 | integration | `cargo test data::sqlite` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-05 | integration | `cargo test data::frequency` | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | DATA-06 | integration | `cargo test data::verification` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `Cargo.toml` — project scaffold with dependencies (macroquad, rusqlite, rand, serde, serde_json, directories, log, env_logger)
- [ ] `src/main.rs` — minimal entry point
- [ ] `tests/` — test infrastructure stubs for grid and data modules

*If none: "Existing infrastructure covers all phase requirements."*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Grid visual inspection | GRID-01 | Grid structure needs human eye to confirm it "looks like" a Dutch crossword | Print grid to stdout, compare against newspaper reference |
| Clue quality spot-check | DATA-03 | Clue "straightforwardness" is subjective | Review 20 random clues at each difficulty level |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
