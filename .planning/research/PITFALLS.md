# Pitfalls Research

**Domain:** Native crossword puzzle game (Rust/macroquad, Dutch/English, European grid style, elderly target user)
**Researched:** 2026-03-21
**Confidence:** MEDIUM (crossword algorithm pitfalls HIGH; macroquad-specific issues MEDIUM; Dutch language specifics MEDIUM)

---

## Critical Pitfalls

### Pitfall 1: Naive Grid Generation Exhausts Search Space Immediately

**What goes wrong:**
A greedy word-placement algorithm that picks the first word that fits will exhaust all options after 3-4 words, even on a 5x5 grid, and loop indefinitely. On a 20x20 European-style grid with a real Dutch dictionary, this becomes a complete non-starter — the algorithm either hangs forever or produces empty/near-empty grids.

**Why it happens:**
Grid generation is NP-complete. Without constraint propagation, each word placement leaves increasingly narrow constraints that no remaining word can satisfy. The search space is exponential and naive backtracking cannot escape local optima.

**How to avoid:**
- Use constraint propagation: after placing each word, immediately prune candidate words for crossing slots to words that match the constrained letter pattern
- Represent candidates as lazy tries/prefix trees, not materialized lists — this avoids memory exhaustion on large dictionaries
- Use heuristics (Most Constrained Variable first): fill the slot with fewest remaining candidates first
- Apply a hard time limit with fallback: if generation exceeds N seconds, restart with different seed or reduced grid density
- Pre-index the word database by pattern (e.g., all 5-letter words with A at position 3) so constraint lookups are O(1)

**Warning signs:**
- Generator takes >2 seconds on a 10x10 grid during development testing
- Grid density drops below 30% fill rate in "completed" grids
- Backtracking counter exceeds thousands of iterations on small grids

**Phase to address:** Grid Generation phase (foundational — must be correct before any other feature builds on it)

---

### Pitfall 2: IJ Digraph Treated as Two Letters Instead of One

**What goes wrong:**
In Dutch crossword puzzles, "IJ" occupies a single grid cell — it is one letter in the Dutch alphabet for crossword purposes. If the grid engine treats IJ as two separate characters, every word containing IJ will have wrong cell counts, broken intersection logic, and incorrect clue number assignments. Words like "IJs" (ice), "rijst" (rice), "vrij" (free) will all malfunction.

**Why it happens:**
UTF-8 string handling iterates over Unicode code points or bytes, not Dutch-alphabet letters. Rust's `str.chars()` gives 'I' and 'J' as separate characters. The digraph only exists in Dutch crossword convention — it is not a Unicode ligature in normal text (U+0132 Ĳ exists but is not used in normal Dutch typography).

**How to avoid:**
- Create a `DutchLetter` abstraction that knows IJ is one "crossword letter" — use it everywhere grid cells, word length, and position are computed
- Normalize all Dutch words at database import time: detect "ij" sequences and tag them so the grid engine knows cell count
- Decide on a canonical representation early (e.g., store as "IJ" token in word data, render as "IJ" in a single cell)
- Never use raw `.len()` or `.chars().count()` on Dutch words without going through this abstraction

**Warning signs:**
- Any word containing "ij" displays with wrong cell count during first rendering test
- Intersection logic fails for crossing words that share an IJ cell

**Phase to address:** Database + word normalization phase (before grid engine touches any Dutch words)

---

### Pitfall 3: AI-Generated Clues Contain Factually Wrong Answers

**What goes wrong:**
Batch-generating clues via the Claude API produces clues where the "answer" implied by the clue does not match the word it is paired with. For example, a clue generated for TAFEL (table) might describe a chair. At difficulty=easy, clues are too vague and match multiple words. At difficulty=hard, clues can be so creative that they are effectively unsolvable. These errors are invisible without human review.

**Why it happens:**
LLMs generate plausible-sounding text that is statistically consistent with the prompt, not logically verified against the answer. When generating thousands of clues in batch, quality degrades. The model may also generate clues in the wrong language (English clue for a Dutch word or vice versa) or produce clues that are too culturally specific.

**How to avoid:**
- After batch generation, run a verification pass: prompt the model with the clue only and ask it to produce the answer — check that the answer matches the target word
- Filter out clues where the model's answer does not match (discard, not fix)
- Sample 50-100 clues manually before deploying the full database — check representative examples at all difficulty levels in both languages
- Generate 3-5 candidate clues per word and keep only the highest-scoring one (self-consistency check)
- Store clue confidence metadata alongside each clue to support the thumbs-up/down feedback loop

**Warning signs:**
- More than 5% of sampled clues, when shown to a human, clearly describe the wrong word
- Clues in Dutch contain English words (or vice versa)
- Easy clues that the target user cannot solve at all after 5 minutes

**Phase to address:** Word/clue database build phase (verification must be part of the generation pipeline, not an afterthought)

---

### Pitfall 4: Grid Interaction UX Fails for Elderly Users

**What goes wrong:**
The click interaction model (click to select, click again to toggle direction) requires short-term memory of previous click state. Elderly users who click slightly off-target or have a slight hand tremor will accidentally toggle direction when they meant to re-select the same cell. The UI shows no visible affordance indicating which direction is currently active, so the user is confused about which word they are filling in.

**Why it happens:**
Developers test with precise mouse control. The double-click-to-toggle pattern works on desktop for young users but the distinction between "first click" and "second click on same cell" becomes unreliable when motor control is impaired.

**How to avoid:**
- Make the active direction immediately obvious with a strong visual indicator (e.g., the highlighted word cells show a directional arrow, or across-word highlighted in one color and down-word in another)
- Show the full active clue text prominently in a fixed panel at all times — not just as a highlighted row in a list
- Consider making direction toggle explicit (a dedicated button or D-key) rather than double-click — simpler mental model
- Add a large, readable "currently entering: [CLUE TEXT]" label that updates whenever direction or selection changes
- Test the interaction model with a real user who is 65+ before finalizing it

**Warning signs:**
- During testing, tester needs to ask "which direction am I typing in?" more than once per session
- Tester types letters that appear in the wrong word

**Phase to address:** Grid interaction phase; verify with real-user testing before UI is considered complete

---

### Pitfall 5: Font Rendering Breaks on Linux Mint at Non-Standard DPI

**What goes wrong:**
macroquad uses fontdue for text rendering. On Linux Mint with a non-standard DPI setting (very common — users scale to 125% or 150% for readability), text renders blurry, too small, or at the wrong size relative to grid cells. On older Linux Mint setups running X11 (not Wayland), HiDPI awareness is limited and macroquad may receive wrong DPI information, causing the UI to be rendered at the wrong scale.

**Why it happens:**
macroquad's windowing backend (miniquad) has partial HiDPI support on X11. `screen_width()` and `screen_height()` return different values depending on whether the window is in fullscreen mode vs windowed. DPI scale factor is not always accurately reported on X11.

**How to avoid:**
- At startup, query `dpi_scale()` and verify it against `screen_width/height` — log both values during development
- Make all font sizes and UI measurements proportional to a single scale factor derived from window dimensions, not hardcoded pixel values
- Test explicitly on a 1920x1080 monitor with 125% scaling and a 4K monitor with 200% scaling
- If macroquad DPI handling proves unreliable, provide a manual scale slider in settings (one setting, not a menu of settings)
- Render text to a texture at known resolution rather than directly if blurriness occurs

**Warning signs:**
- Grid cells and font sizes look correct on developer machine but wrong in a screenshot taken by the target user
- Any text rendering test fails at 125% display scale on X11

**Phase to address:** Rendering/layout phase (must test on actual Linux Mint hardware before declaring complete)

---

### Pitfall 6: Flatpak Build Breaks Because of Cargo's Network Dependency

**What goes wrong:**
Flatpak builds are performed in a sandboxed offline environment. `cargo build` cannot reach crates.io during the Flatpak build process. If the manifest does not correctly pre-bundle all Cargo dependencies (including transitive dependencies), the build fails with network errors. Any Cargo.toml git dependency (instead of crates.io version) will fail silently or with a cryptic error.

**Why it happens:**
Flatpak sandbox denies all network access during build by default. Developers who build normally with `cargo build` never encounter this because they have internet access. The tooling (`flatpak-cargo-generator.py`) can fail to capture git dependencies or workspace members.

**How to avoid:**
- Add the Flatpak build step early in development — do not leave it to the final "polish" phase
- Use `flatpak-cargo-generator.py` from the official `flatpak-builder-tools` repository to generate `cargo-sources.json` from `Cargo.lock`
- Set `CARGO_NET_OFFLINE: 'true'` in the Flatpak build manifest
- Avoid git-sourced Cargo dependencies entirely — pin all dependencies to crates.io versions
- Test the Flatpak build in CI before any release

**Warning signs:**
- Any `Cargo.toml` dependency using `git = "..."` instead of `version = "..."`
- Flatpak build fails with "network" or "offline mode" errors
- `cargo-sources.json` is not regenerated after updating `Cargo.lock`

**Phase to address:** Distribution/packaging phase, but dependency hygiene must be enforced from the start

---

### Pitfall 7: European Grid Generator Produces Isolated Word Islands

**What goes wrong:**
The generator places words without enforcing connectivity of the white-cell region. The resulting grid has clusters of words that are not connected to each other via any path of white cells — effectively two separate sub-puzzles on one grid. The puzzle looks valid (words have intersections) but the overall grid is fragmented.

**Why it happens:**
Word placement algorithms focus on letter-intersection constraints and do not check global grid connectivity. European-style grids (with high black-square density) are especially prone to fragmentation because the generator has more freedom to place black squares.

**How to avoid:**
- After every grid is generated, run a flood-fill from any white cell and verify all other white cells are reachable
- Reject and regenerate any grid that fails the connectivity check
- Alternatively, bias word placement toward the center of the grid to prevent edge clustering
- Track which regions of the grid are "connected to the main body" during generation and refuse to start a new region

**Warning signs:**
- Visual inspection of any generated grid shows isolated word groups in corners
- Flood-fill test implemented as a unit test fails on >5% of generated grids

**Phase to address:** Grid generation phase (connectivity check is a mandatory acceptance criterion for any generated grid)

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Hardcode font sizes in pixels | Fast to implement | Breaks at non-standard DPI; requires full layout rework | Never — use scale factor from day one |
| Generate clues without verification pass | Faster database build | Users encounter wrong-answer clues; erodes trust in the game | Never — verification is cheap to add to the pipeline |
| Store puzzle state as plain text file | Simple to implement | Fragile to format changes after updates; state lost on corruption | Acceptable for MVP if format version number is stored |
| Treat IJ as two letters for now | Avoids early complexity | Impossible to retrofit later without rewriting the grid engine | Never — must be designed in from first grid cell implementation |
| Use a simple word frequency list without crossword-specific filtering | Fast to get a word database | Grid will contain acronyms, hyphenated words, proper nouns, offensive words | Never for Dutch; use a curated filter from the start |
| Skip Flatpak build until end | Faster dev iteration | Dependency issues discovered late, may require dependency changes | Acceptable during early prototyping only |
| Linear scan word database per placement | Simple code | 10+ second generation times on 20x20 grids | Only acceptable for a 5x5 proof-of-concept |

---

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Claude API clue generation | Sending all words in one giant prompt — output quality degrades badly for large batches | Batch in groups of 20-50 words; include explicit format constraints in the prompt |
| Claude API clue generation | Not specifying that the answer IS the word in the prompt — model invents a different answer | Make the prompt explicit: "Generate a clue for the Dutch word TAFEL. The answer to the clue must be exactly TAFEL." |
| Claude API clue generation | Generating clues in bulk without a post-hoc answer-verification pass | Add a verification step: ask the model to answer its own clue; discard mismatches |
| Flatpak cargo bundling | Running `flatpak-cargo-generator.py` once and forgetting to re-run it after updating dependencies | Re-run the generator and commit the updated `cargo-sources.json` every time `Cargo.lock` changes |
| macroquad on Linux Wayland | Using XWayland compatibility layer — causes blurry rendering and scaling issues | Set the correct `WINIT_UNIX_BACKEND=wayland` env var or configure Flatpak to run natively on Wayland |
| Word database at build time | Storing the raw LLM output directly without canonicalization | Normalize case, strip punctuation, validate word length, and remove duplicates before writing the database |

---

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Linear word lookup per grid slot | Grid generation takes minutes on 20x20 | Pre-index words by pattern (length + letter constraints) in a HashMap | As soon as the grid reaches ~10x10 with a real dictionary |
| Materializing all valid row configurations | Memory exhaustion during generation | Use lazy iterators / trie traversal | Immediately on grids larger than 7x7 with vocabulary >10,000 words |
| Rendering the full word list without virtualization | Frame rate drops when clue list is long | Only render visible clue items (scroll viewport), not all 60+ clues | With 40+ clues on a 20x20 grid |
| Re-scanning word history on every puzzle generation | Slow puzzle start | Keep a HashSet of recently used words in memory; persist to disk only on quit | When history grows beyond 500 words |
| Loading the full word database at startup | Slow cold start (elderly users notice 5+ second delays) | Load eagerly at startup but show a splash or progress indicator; or use memory-mapped file | On lower-end Linux machines with a large Dutch word list (>50,000 words) |

---

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Small font size in clue list | 70-year-old user cannot read clues without glasses — abandons game | Minimum 18pt equivalent for clue text; 24pt preferred; make it the default, not an accessibility option |
| No indication of current word direction | User types letters into wrong word without realizing | Show the active clue text prominently above the grid; use strong color differentiation for across vs down selection |
| Filled cells look identical to empty cells at a glance | User loses track of progress | Use strong contrast between filled (character visible) and empty (clearly blank) cells; never rely on color alone |
| Error state (wrong letter) not shown until puzzle complete | User cannot self-correct — frustrating | Show incorrect cells (light red background) or provide a "check my answers" button that highlights errors |
| Puzzle completion message easy to miss | User does not realize they have finished | Show a full-screen or large modal congratulations — not just a small status bar update |
| Clue list scrolls independently from grid | User must context-switch constantly | When a cell is selected, auto-scroll the clue list to show that clue; keep grid and clue list in sync |
| No way to clear a cell | User cannot erase a wrong letter | Backspace clears current cell and moves backward; Delete clears without moving; document this visually |
| Difficulty level names unclear | "Medium" means different things to different people | Add a one-sentence description under each difficulty name: "Easy: common words, descriptive clues" |

---

## "Looks Done But Isn't" Checklist

- [ ] **Grid generation:** Connectivity of white cells verified — flood-fill test passes on 100 consecutive generated grids
- [ ] **Dutch IJ handling:** Words containing "ij" display in a single cell; word length counts correctly; crossing words intersect at the right position
- [ ] **Clue database:** Verification pass run on clues — answer-back check confirms each clue points to the right word
- [ ] **Dutch/English language switch:** Clue language switch mid-puzzle shows clues in the new language without resetting grid state
- [ ] **Word history:** Words used in recent N puzzles do not appear in newly generated puzzle — test by generating 10 consecutive puzzles
- [ ] **State persistence:** Quit mid-puzzle and relaunch — puzzle resumes with all filled cells and selection state intact
- [ ] **Flatpak build:** Flatpak package builds cleanly in offline mode from `cargo-sources.json` — test in a clean build environment
- [ ] **Font scaling:** All text is readable at 125% and 150% Linux display scale settings on a real Linux Mint machine
- [ ] **Clue feedback persistence:** Thumbs-up/down rating is written to disk and survives app restart
- [ ] **Completion detection:** Puzzle marked complete only when all cells are correctly filled — not when all cells are filled (allowing wrong answers)

---

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Grid engine does not handle IJ | HIGH | Requires rewriting cell representation, word indexing, and intersection logic — touches entire codebase |
| AI-generated clues contain wrong answers | LOW-MEDIUM | Re-run verification pass and regenerate failed clues; can be done without touching the game code |
| Flatpak build fails due to git dependencies | MEDIUM | Replace git deps with crates.io versions; regenerate cargo-sources.json; may require finding alternatives if no crates.io version exists |
| Grid produces disconnected islands | MEDIUM | Add post-generation connectivity check + rejection; increase word placement attempts before giving up |
| Font rendering broken at target user's DPI | MEDIUM | Introduce scale factor parameter; rebuild UI layout to use it everywhere — affects all rendering code |
| State file format incompatible after update | LOW | Add format version field from day one; migration code for each version transition |
| Naive search hangs on 20x20 grid | HIGH | Requires replacing the core generation algorithm; everything built on top must be retested |

---

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Naive grid generation exhausts search space | Grid generation phase | Generator produces a 20x20 grid in <5 seconds reliably |
| IJ treated as two letters | Database normalization + grid engine foundation | Unit test: Dutch word "vrij" occupies 4 cells, not 5 |
| AI clues contain wrong answers | Clue database build phase | Answer-back verification pass shows <2% mismatch rate |
| Grid interaction fails elderly users | Grid interaction phase | Real-user test with 65+ person: no confusion about direction |
| Font rendering broken at non-standard DPI | Rendering/layout phase | Tested on Linux Mint at 125% scale — all text readable |
| Flatpak offline build fails | Distribution phase (but dependency hygiene from day 1) | Clean-room Flatpak build succeeds from cargo-sources.json |
| Grid produces isolated word islands | Grid generation phase | Flood-fill connectivity test passes on all generated grids |
| Clue database has wrong-difficulty clues | Clue database build phase | Sample review: easy clues solvable in <3 min; hard clues require >10 min |

---

## Sources

- Algorithmic crossword generation practical experience: https://blog.eyas.sh/2025/12/algorithmic-crosswords/
- Crossword generation NP-completeness and trie-based approaches: https://www.mdpi.com/1999-4893/15/1/22
- Constructing crossword grids — heuristics vs constraints: https://www.researchgate.net/publication/2352510_Constructing_Crossword_Grids_Use_of_Heuristics_vs_Constraints
- IJ digraph in Dutch crosswords: https://en.wikipedia.org/wiki/IJ_(digraph)
- AI-generated clue quality issues: https://www.alibaba.com/product-insights/why-do-ai-generated-crossword-clues-feel-unsatisfying-and-how-to-constrain-outputs-for-fair-misdirection-and-linguistic-elegance
- UX for elderly users: https://pmc.ncbi.nlm.nih.gov/articles/PMC12350549/ and https://www.eleken.co/blog-posts/examples-of-ux-design-for-seniors
- macroquad modifier keys macOS bug (known issue): https://github.com/not-fl3/macroquad/issues/429
- macroquad screen size vs fullscreen bug: https://github.com/not-fl3/macroquad/issues/237
- macroquad single-maintainer bus factor: https://games.brettchalupa.com/devlog/the-state-of-game-dev-in-rust-2024/
- Flatpak Rust packaging (cargo offline build): https://belmoussaoui.com/blog/8-how-to-flatpak-a-rust-application/
- flatpak-cargo-generator tool: https://github.com/flatpak/flatpak-builder-tools/tree/master/cargo
- Flatpak sandbox security and permissions pitfalls: https://www.linuxjournal.com/content/when-flatpaks-sandbox-cracks-real-life-security-issues-beyond-ideal
- KDE Rust Flatpak guide: https://develop.kde.org/docs/getting-started/rust/rust-flatpak/

---
*Pitfalls research for: Native crossword puzzle game (Puuzel) — Rust/macroquad, Dutch/English, European grid style*
*Researched: 2026-03-21*
