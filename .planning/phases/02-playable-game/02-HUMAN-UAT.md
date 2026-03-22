---
status: partial
phase: 02-playable-game
source: [02-VERIFICATION.md]
started: 2026-03-22T10:00:00Z
updated: 2026-03-22T10:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Full game flow visual verification
expected: Menu shows Puuzel title + three buttons. After difficulty select, loading screen appears. Grid renders with black cells, white cells, clue numbers, two-panel layout. Cell click highlights cell in blue, same-cell click toggles direction, active word highlighted in light blue. Typing fills cell and advances. Backspace clears. Clue panel shows Horizontaal/Verticaal sections with scrollable clues, clicking a clue jumps cursor. Completing puzzle triggers Gefeliciteerd! overlay with Nieuwe puzzel button.
result: [pending]

### 2. Double-click rating dialog
expected: Double-clicking a word (same cell within 300ms) shows a small dialog with clue text, Goed and Slecht buttons. Clicking Goed or Slecht dismisses the dialog.
result: [pending]

### 3. Clue panel font size adequacy (DISP-02)
expected: Clue text at 15px is legible for a 70-year-old user without squinting at 1280x800.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
