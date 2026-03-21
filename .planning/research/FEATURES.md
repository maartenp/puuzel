# Feature Research

**Domain:** Desktop crossword puzzle game (Dutch/European grid style, dual language)
**Researched:** 2026-03-21
**Confidence:** MEDIUM — Grid interaction patterns from app research are HIGH; Dutch-specific grid conventions are MEDIUM (sparse authoritative sources, inferred from European conventions)

---

## Dutch/European Grid Conventions (Reference)

Understanding conventions is prerequisite to knowing which features are table stakes vs. unique to this project.

### American NYT-Style
- Every letter is "checked" (part of both an across and down word)
- 180-degree rotational symmetry required
- Minimum 3-letter words enforced
- Black square density ~16% (1/6 of grid)
- No unchecked letters permitted

### British/Cryptic-Style
- ~50% of letters are unchecked (alternate-letter pattern)
- Higher black square density (~25%)
- No 2-letter words
- Symmetry maintained

### Dutch/European-Style (this project's target)
- **Unchecked letters permitted** — not every letter needs to be in both an across and down word. This is the defining difference from American style.
- **Two-letter words permitted** — shorter words can appear in the grid.
- **No mandatory rotational symmetry** — black square placement is more flexible.
- **Higher black square density** than American style, closer to British in appearance.
- **IJ digraph** is treated as a single letter occupying one cell (Dutch-specific).
- **Connected white squares** — all white cells must form one contiguous region (universal convention).
- **Grid size**: Typically larger than American puzzles; ~20x20 is conventional for Dutch newspaper crosswords.
- **Clue style**: Straightforward definitions, not cryptic wordplay (unlike British cryptics).

---

## Feature Landscape

### Table Stakes (Users Expect These)

These features are expected by anyone who has used a digital crossword. Missing them makes the app feel broken.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Render crossword grid on screen | Core product | MEDIUM | macroquad canvas; grid must be pixel-accurate, readable |
| Display numbered clue list (across/down) | Every crossword interface has this | LOW | Scrollable, synchronized with grid selection |
| Click cell to select it | Universal input metaphor | LOW | Must handle grid coordinate mapping |
| Highlight selected word's cells | Shows which word is active | LOW | Color fill or border highlight on all cells of active word |
| Keyboard input fills selected cell | Core interaction loop | LOW | Type a letter, cell fills, cursor advances |
| Auto-advance cursor after letter entry | Expected by anyone who's used a crossword app | LOW | Move to next empty cell in current word direction |
| Backspace/Delete clears current cell | Correction mechanic | LOW | If cell empty, move backwards and clear |
| Toggle across/down direction | Needed to switch between intersecting words | LOW | Click same cell again to toggle; or dedicated key |
| Clicking a clue highlights its word | Clue-to-grid navigation | LOW | Selects word, highlights cells, positions cursor at first open cell |
| Detect puzzle completion | Game loop closure | LOW | Check all cells filled correctly, trigger congratulation |
| Congratulations/completion screen | Positive reinforcement | LOW | Simple message, option to start new puzzle |
| New puzzle generation | Core value proposition | HIGH | Must produce valid Dutch-style grid with connected cells |
| State persistence (resume on relaunch) | Users don't want to lose progress | MEDIUM | Serialize grid state to disk; load on startup |
| Large readable fonts | Expected by target demographic; also general good practice | LOW | macroquad font rendering; minimum comfortable size |
| High contrast display | Accessibility baseline | LOW | White-on-black as specified; good for elderly users |

### Differentiators (Competitive Advantage)

Features that distinguish Puuzel from generic crossword apps or printable newspaper puzzles.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| On-demand puzzle generation | Fresh puzzles whenever wanted; no daily limit, no subscription | HIGH | Core differentiator vs. NYT/daily apps; requires valid Dutch-style grid generation algorithm |
| Dutch language support | No mainstream desktop app does Dutch crosswords well | MEDIUM | Requires Dutch word+clue database |
| Three difficulty levels | Adapts to mood and skill; beginner-friendly | MEDIUM | Affects word commonness, word length, grid density, clue complexity |
| Bilingual clue display | Solve Dutch words with English clues (or vice versa) for language learning | MEDIUM | Six clue variants per word (3 difficulties x 2 languages); switchable mid-puzzle |
| In-puzzle clue language switching | Unique feature; bridges Dutch/English for bilingual users | LOW | Toggle language on active clue list; clues already pre-generated |
| Clue quality feedback (thumbs up/down) | Enables improvement loop; user feels heard | LOW | Double-click word to rate; persist ratings locally |
| Word history tracking | Reduces repetition; keeps puzzles fresh | MEDIUM | Track last N puzzles' words; exclude from generation |
| Dutch/European grid style | Matches what target user knows from decades of newspaper puzzles | HIGH | The grid generator itself is the differentiator |
| Flatpak distribution with auto-update | Zero-friction updates for non-technical Linux user | MEDIUM | Packaging concern, not gameplay |
| Single-user, no account required | Simplified experience; no login friction | LOW | No network dependency during gameplay |

### Anti-Features (Deliberately Not Building)

Features that look attractive but would harm this product for this user.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Timer / countdown clock | Common in competitive crossword apps | Creates anxiety; wrong for a 70-year-old user solving for pleasure; pressure ≠ fun | Offer session time *display* (informational) only if ever requested |
| Streak tracking | Gamification; keeps users coming back | Punishes missed days; creates obligation; turns leisure into task | Just make each puzzle good; no meta-game needed |
| Puzzle history / statistics dashboard | Users like seeing progress | Adds complexity; clutters UI; irrelevant for single-user casual play | Clue feedback ratings are the only data worth keeping |
| Social / multiplayer / sharing | Sharing is a natural impulse | Single-player product; adds server infra, auth, significant scope | Out of scope; desktop-only, single-player |
| In-app clue editing | Power users want to fix bad clues | Creates complexity; moderation burden; thumbs up/down is sufficient signal | Offline curation pipeline using collected rating data |
| Cryptic / wordplay clues | British-style puzzle fans | Wrong genre for target user; confusing for non-native speakers | Straightforward definition clues only |
| NYT-style symmetric grid | American crossword fans | Doesn't match Dutch newspaper conventions the target user knows | Dutch/European grid style is the correct choice |
| Online puzzle sync / cloud save | Multi-device users | Requires internet, account, server; adds complexity and failure modes | Local file persistence is sufficient for single desktop |
| Hints / reveal letter / reveal word | Players who get stuck | Degrades puzzle satisfaction; easier to just start a new puzzle since generation is instant | "Start new puzzle" replaces hint need when generation is free |
| Pencil mode (tentative letters) | Power solvers who like to guess tentatively | Adds UI complexity; target user unlikely to use or understand it | Simple: type a letter, it appears; backspace to remove |
| Animations / transitions | Modern app feel | Can confuse or distract elderly users; adds rendering complexity | Clean, static, immediate UI changes only |
| Mobile / tablet support | Wider audience | Touch targets, keyboard handling, screen rotation all require redesign; out of scope | Desktop-only is the right constraint for now |

---

## Feature Dependencies

```
[Puzzle Generation]
    └──requires──> [Word+Clue Database] (Dutch and/or English)
    └──requires──> [Grid layout algorithm] (valid Dutch-style grid)
    └──requires──> [Word history tracking] (to avoid repetition)

[Grid Interaction]
    └──requires──> [Grid render] (cells visible on screen)
    └──requires──> [Cell selection model] (what is selected)
        └──requires──> [Direction state] (across vs. down)

[Clue List Display]
    └──requires──> [Grid layout] (clue numbers come from layout)
    └──enhances──> [Cell selection] (click clue highlights cells)

[Bilingual clue display]
    └──requires──> [Word+Clue Database with 6 variants per word]
    └──enhances──> [Difficulty levels] (each language has 3 difficulty tiers)

[Clue quality feedback]
    └──requires──> [Grid interaction] (double-click a word)
    └──requires──> [Persistence layer] (ratings saved to disk)

[State persistence]
    └──requires──> [Grid interaction] (something worth saving)
    └──requires──> [Persistence layer] (disk I/O)

[Puzzle completion detection]
    └──requires──> [Grid interaction] (letters filled in)
    └──requires──> [Word+Clue Database] (correct answers to check against)

[Clue language toggle]
    └──requires──> [Bilingual clue variants loaded]

[Word history tracking]
    └──requires──> [Persistence layer]
    └──enhances──> [Puzzle generation] (exclusion list fed to generator)
```

### Dependency Notes

- **Puzzle generation requires the word+clue database**: The database must exist and be bundled before the generator can produce any puzzle. This is Phase 1 groundwork.
- **All grid interaction requires grid render**: Rendering is the foundation; no interaction without it.
- **Bilingual clues require six-variant database**: The clue generation pipeline (3 difficulties x 2 languages) must be complete before bilingual switching works. The mid-puzzle language toggle is trivially easy once the data exists.
- **Persistence layer is shared**: State persistence, clue ratings, and word history all write to local storage. Build it once, reuse for all three.
- **Puzzle completion requires correct answers**: The generator must store the solution alongside the blank grid; completion detection is a simple comparison.

---

## MVP Definition

### Launch With (v1)

Minimum viable product: the target user can sit down, start a puzzle, fill in answers, and feel satisfied.

- [ ] Dutch word+clue database bundled (at least one difficulty level, one language) — without this nothing works
- [ ] Valid Dutch/European-style grid generation (~20x20) — core value proposition
- [ ] Grid rendered on screen with clue list — visible, readable, high contrast
- [ ] Cell selection (click to select, click again to toggle direction) — core input
- [ ] Keyboard input with auto-advance — filling in letters
- [ ] Clue click highlights word — navigation between clue list and grid
- [ ] Backspace/delete to correct mistakes — basic error correction
- [ ] Puzzle completion detection + congratulations — satisfying loop closure
- [ ] State persistence (resume on relaunch) — prevents frustration at accidental close
- [ ] "New puzzle" button — start fresh

### Add After Validation (v1.x)

Features that make the product better but aren't needed to validate the core.

- [ ] Three difficulty levels — add once base generation is proven; easy to parameterize
- [ ] English word+clue database — extend database after Dutch is working
- [ ] Bilingual clue display with in-puzzle language toggle — nice but not needed to enjoy one language
- [ ] Clue quality feedback (thumbs up/down) — add once user is regularly solving puzzles
- [ ] Word history tracking — add once user has completed enough puzzles for repetition to be noticeable
- [ ] Flatpak packaging and auto-update — distribution concern; add after gameplay is solid

### Future Consideration (v2+)

- [ ] macOS DMG packaging — secondary platform; add after Linux is stable
- [ ] English-as-primary-language mode (English words, English clues) — currently the design is Dutch-first
- [ ] Additional grid sizes (small/quick puzzles) — currently scoped to ~20x20

---

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Dutch word+clue database | HIGH | HIGH | P1 |
| Dutch/European grid generation | HIGH | HIGH | P1 |
| Grid render + clue list display | HIGH | MEDIUM | P1 |
| Cell selection + direction toggle | HIGH | LOW | P1 |
| Keyboard input + auto-advance | HIGH | LOW | P1 |
| Backspace/error correction | HIGH | LOW | P1 |
| Clue click → highlight word | HIGH | LOW | P1 |
| Puzzle completion + congratulations | HIGH | LOW | P1 |
| State persistence | HIGH | MEDIUM | P1 |
| Three difficulty levels | MEDIUM | MEDIUM | P2 |
| English word+clue database | MEDIUM | HIGH | P2 |
| Bilingual clue display + toggle | MEDIUM | LOW (given data) | P2 |
| Word history tracking | MEDIUM | LOW | P2 |
| Clue quality feedback | LOW | LOW | P2 |
| Flatpak packaging + auto-update | HIGH (for distribution) | MEDIUM | P2 |
| macOS packaging | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch (MVP)
- P2: Should have; add after MVP is working
- P3: Nice to have; future consideration

---

## Competitor Feature Analysis

| Feature | NYT Crossword App | Penny Dell App | Puuzel Approach |
|---------|------------------|----------------|-----------------|
| Puzzle supply | Daily editorial puzzle | Curated puzzle packs | On-demand generated; infinite supply |
| Language | English only | English only | Dutch + English |
| Grid style | American (fully checked, symmetric) | American | Dutch/European (unchecked ok, no symmetry requirement) |
| Difficulty | Fixed per puzzle | By pack selection | 3 levels on demand |
| Error checking | Check letter/word/puzzle | Show/hide errors | Not planned (start new puzzle instead) |
| Hints / reveal | Reveal letter/word/puzzle | Hint-supplied letters | Not planned |
| Pencil mode | No | Yes | No |
| Timer | No (was removed) | Optional | No |
| Streaks | Yes | No | No |
| Subscription | Required | Paid packs | No (bundled data, one-time) |
| Platform | Mobile | Mobile | Desktop (Linux, macOS) |
| Clue language toggle | No | No | Yes (unique feature) |
| Clue quality feedback | No | No | Yes (thumbs up/down) |
| State persistence | Yes | Yes | Yes |
| Word history | N/A (editorial) | N/A (editorial) | Yes (generated puzzles) |

---

## Sources

- [Crossword - Wikipedia](https://en.wikipedia.org/wiki/Crossword) — grid style conventions overview
- [CommuniCrossings: Crossword Types](https://communicrossings.com/crosswords-terminology-types) — European vs. American grid terminology
- [CommuniCrossings: Grid Construction](https://communicrossings.com/constructing-crosswords-grid) — American grid rules (contrast reference)
- [Always Puzzling: American vs British Crosswords](https://alwayspuzzling.blogspot.com/2013/01/american-vs-british-crosswords.html) — checked/unchecked letter conventions
- [Crossword Unclued: Grid Checking](https://www.crosswordunclued.com/2009/09/crossword-grid-checking.html) — checking conventions detail
- [Penny Dell Crossword App](https://www.pennydellpuzzles.com/crosswordapp/) — feature set reference for table stakes
- [Crossword User Guide (APH)](https://tech.aph.org/pz/) — accessibility and feature conventions
- [Make Tech Easier: Best Crossword Apps](https://www.maketecheasier.com/best-android-crossword-apps-for-word-enthusiasts/) — app feature comparison
- [Dutch Wikipedia: Kruiswoordpuzzel](https://nl.wikipedia.org/wiki/Kruiswoordpuzzel) — Dutch crossword conventions (attempted; 403 error; conventions inferred from other sources)
- [Dutch crossword rules (regels.nl)](https://www.regels.nl/spelletjes/kruiswoord/) — Dutch puzzle conventions
- [DailyCaring: Crossword Puzzles for Seniors](https://dailycaring.com/free-large-print-crossword-puzzles-for-seniors/) — elderly user accessibility considerations

---

*Feature research for: Puuzel — Dutch/European crossword puzzle desktop app*
*Researched: 2026-03-21*
