---
phase: 02
slug: playable-game
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-21
---

# Phase 02 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | PGEN-01, PGEN-05, DISP-01 | integration | `cargo test` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | INTR-01, INTR-02, INTR-03, INTR-04 | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | DISP-02, DISP-03, DISP-04 | manual | N/A — visual | N/A | ⬜ pending |
| 02-02-02 | 02 | 1 | INTR-05, INTR-06, INTR-07 | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 2 | FLOW-01, FLOW-02 | unit | `cargo test` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 2 | INTR-08, INTR-09, DISP-05 | manual | N/A — visual/interactive | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test infrastructure already exists (cargo test, 43 tests passing)
- [ ] Game state machine tests will be added per-task
- [ ] Input handling tests will use unit tests on state transitions (not UI rendering)

*Existing infrastructure covers compilation and unit test needs. Visual rendering requires manual verification.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Grid renders with correct cell sizes and colors | DISP-01, DISP-03, DISP-04 | Visual rendering output | Launch app, verify grid fills screen, cells are large, high contrast |
| Font readability for elderly user | DISP-02 | Subjective visual quality | Launch app, verify text is large and legible at arm's length |
| Clue list scrolls and highlights active clue | DISP-05, INTR-07 | Interactive UI behavior | Click cells, verify clue panel follows selection |
| Congratulations screen appears on completion | FLOW-01 | End-to-end game flow | Complete a puzzle, verify congratulations overlay |
| IJ digraph displays correctly in grid cell | GRID-03 (visual) | Visual rendering | Generate puzzle with IJ word, verify single-cell display |

*Note: macroquad rendering is immediate-mode — unit tests verify state, manual tests verify visual output.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
