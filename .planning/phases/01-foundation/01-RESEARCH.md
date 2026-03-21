# Phase 1: Foundation - Research

**Researched:** 2026-03-21
**Domain:** Crossword grid generation (Dutch/European style) + Dutch word/clue database pipeline
**Confidence:** HIGH (stack decisions confirmed against crates.io; algorithm patterns verified from multiple sources)

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Difficulty calibration:**
- D-01: All difficulties use 20x20 grids — no size variation
- D-02: Black square ratio: easy ~35-40%, medium ~30-35%, hard ~25-30% (European newspaper range)
- D-03: LLM rates each clue with a difficulty level during generation; difficulties pick from their rated pool
- D-04: LLM also rates word commonness — easy puzzles avoid obscure words entirely
- D-05: Difficulty calibration is iterative — v1 ships with LLM-rated defaults, users rate completed puzzles 1-10 (prompted after completion) to guide future tuning
- D-06: Difficulty rating is internal only — never shown to the user

**Word selection criteria:**
- D-07: No abbreviations in the word list
- D-08: Proper nouns allowed — place names (Amsterdam, Rijn) and people (Rembrandt, Cruijff) are fair game
- D-09: Maximum word length: 15 characters
- D-10: Minimum word length: 2 characters (European convention permits two-letter words)
- D-11: Auto-filter vulgar/offensive words from the word list
- D-12: Archaic/literary Dutch words included if clue explicitly mentions archaic (e.g., "Ouderwets woord voor 'graag'")
- D-13: Compound words welcome up to the 15-char cap

**Clue style per difficulty:**
- D-14: Easy clues: direct synonyms or conversational definitions (both styles acceptable)
- D-15: Hard clues: indirect/lateral definitions requiring inference or specific knowledge
- D-16: Medium clues: factual but requiring more thought than easy — sits between direct and lateral
- D-17: Maximum clue length: 6 words
- D-18: All clues are straightforward definitions — no cryptic or wordplay clues
- D-19: Clues generated in Dutch only for v1

**IJ digraph handling:**
- D-24: IJ occupies a single cell in the grid — "IJSBEER" = 6 cells (IJ-S-B-E-E-R)
- D-25: Cell displays "IJ" (two ASCII characters), not the Unicode ligature
- D-26: Input: user presses I then J (no other key in between) → merges into single IJ cell
- D-27: No time limit on the I→J sequence — purely sequential keypress detection
- D-28: Backspace on an IJ cell: first press reverts to "I" in the cell, second press clears it
- D-29: Words in the database store IJ as a single token for grid-fitting purposes (counts as 1 letter toward grid placement)

### Claude's Discretion

- Grid generation algorithm choice (constraint propagation, backtracking strategy)
- OpenTaal word list filtering pipeline (inflection removal, frequency scoring methodology)
- Claude API prompt design for clue generation and self-verification
- SQLite schema design for word/clue storage
- Word frequency/commonness scoring approach
- IJ detection in the OpenTaal source list (normalization strategy)

### Deferred Ideas (OUT OF SCOPE)

- LLM-driven clue quality analysis from user feedback data — v2 feature
- English word+clue database (SCOWL + AI clues) — v2 (LANG-01 through LANG-04)
- Bilingual clue switching — v2
- Puzzle rating 1-10 data analysis and difficulty rebalancing — v2
- Clue feedback pattern detection via LLM — v2
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GRID-01 | App generates valid Dutch/European-style crossword grids (~20x20) | CSP with backtracking algorithm; slot-based word placement |
| GRID-02 | Generated grids have connected white squares (one contiguous region) | Flood-fill connectivity check after grid construction |
| GRID-03 | IJ digraph is treated as a single cell in the grid | Token normalization in word table; grid Cell enum variant |
| GRID-04 | Unchecked letters are permitted (not every letter needs both across and down) | European style explicitly allows unches — no enforcement needed |
| GRID-05 | Two-letter words are permitted in the grid | min_word_length = 2 in generator config |
| GRID-06 | Black square density varies by difficulty (easy = more black squares, hard = fewer) | D-02: 35-40% / 30-35% / 25-30%; density-controlled black square seeding |
| GRID-07 | Word length varies by difficulty (easy = shorter average, hard = longer) | Word length filter per difficulty tier in word selection query |
| GRID-08 | Word commonness varies by difficulty (easy = everyday words, hard = less common) | Frequency/commonness score in DB schema; query filter per difficulty |
| DATA-01 | Dutch word list sourced and filtered for crossword suitability | OpenTaal basiswoorden-gekeurd.txt + basiswoorden-ongekeurd.txt (proper nouns); filter pipeline documented below |
| DATA-02 | AI-generated clues for each word at three difficulty levels (easy, medium, hard) | Claude Batch API; one generation request per word covering all 3 difficulty levels |
| DATA-03 | Clues are straightforward definitions (not cryptic or wordplay) | Prompt constraint per D-18 |
| DATA-04 | Word+clue database bundled with app in SQLite format | rusqlite 0.39.0 with `bundled` feature |
| DATA-05 | Database includes word frequency/commonness metadata for difficulty filtering | commonness_score INTEGER (1-5) in words table; rated by LLM during clue generation |
| DATA-06 | AI clue generation includes self-verification pass (model answers its own clue to validate) | Two-step prompt: generate clue → verify by answering it; reject if answer doesn't match |
</phase_requirements>

---

## Summary

Phase 1 delivers two independent artifacts: (1) a Rust crate that generates 20x20 Dutch/European-style crossword grids, and (2) an offline build pipeline that produces a bundled SQLite database of Dutch words with AI-generated clues.

The grid generator should use a slot-based Constraint Satisfaction Problem (CSP) approach with backtracking and arc consistency pruning. Research confirms this architecture can generate 20x20 grids in well under 10 seconds on modern hardware when paired with a pre-indexed word lookup structure (either a trie or a position-letter bitmap index). The Dutch/European grid style is less constrained than American style — no symmetry requirement, unchecked letters (unches) are normal, two-letter words allowed — which actually makes generation easier.

The database pipeline is a sequential offline script (not part of the game binary): download OpenTaal wordlist → filter → normalize IJ → batch-generate clues via Claude Batches API → self-verify clues → write to SQLite. The database is then committed to the repo or generated at build time. All runtime API calls are forbidden by project constraints.

**Primary recommendation:** Build the grid generator as a pure Rust library crate with no macroquad dependency; the database pipeline is a standalone Python or shell script run at build time. Keep the two artifacts cleanly separated so Phase 2 (rendering) can consume the grid engine without touching the data pipeline.

---

## Standard Stack

### Core (verified against crates.io on 2026-03-21)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust stable | 1.85+ | Language | Non-negotiable per project constraints |
| rusqlite | **0.39.0** | Bundled SQLite for word/clue DB | `bundled` feature embeds SQLite 3.51.3; no system dep; works in Flatpak sandbox |
| rand | 0.10.0 | RNG for word selection + grid shuffle | Project-standard; `rng()` API (not deprecated `thread_rng()`) |
| serde + serde_json | 1.x | Serialize grid state; interchange format for pipeline output | Project-standard |
| anthropic Python SDK (or HTTP) | latest | Build-time clue generation script | Build-time only; not a Rust dep |

> **IMPORTANT:** CLAUDE.md lists rusqlite 0.32 but the current version is **0.39.0** (confirmed via `cargo search` on 2026-03-21). Use 0.39.0. The `bundled` feature and API are unchanged — only the version number needs updating in Cargo.toml.

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| directories | 6.0.0 | XDG-compliant save paths | Phase 2+; not needed for Phase 1 pure-library work |
| log + env_logger | 0.4.x / 0.11.x | Generator debug output | Add from the start; helps debug stuck generation |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Hand-rolled trie | `radix_trie` crate (0.2.x) | Radix trie saves memory on large Dutch dictionary; hand-rolled is simpler and sufficient if the position-letter bitmap index is used instead |
| rusqlite bundled | rusqlite non-bundled | Non-bundled fails in Flatpak sandbox — never use |
| Claude Batches API | Claude streaming API one-by-one | Batches API is 50% cheaper and designed for exactly this bulk offline use case |

**Installation (Cargo.toml additions for Phase 1):**
```toml
[dependencies]
rusqlite = { version = "0.39", features = ["bundled"] }
rand = "0.10"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
log = "0.4"
env_logger = "0.11"
```

**Version verification:** Confirmed on 2026-03-21:
- `rusqlite` → 0.39.0 (via `cargo search rusqlite`)
- `rand` → 0.10.0 (via `cargo search rand`)
- `macroquad` → 0.4.14 (via `cargo search macroquad`) — not needed in Phase 1 grid library

---

## Architecture Patterns

### Recommended Project Structure

```
puuzel/
├── Cargo.toml                  # workspace or single-crate
├── src/
│   ├── main.rs                 # macroquad entry (Phase 2+); minimal in Phase 1
│   ├── grid/
│   │   ├── mod.rs              # public API: Grid, Cell, Direction, Slot
│   │   ├── generator.rs        # CSP backtracking engine
│   │   ├── connectivity.rs     # flood-fill white-square check
│   │   └── difficulty.rs       # DifficultyConfig (density, word length bounds)
│   └── db/
│       ├── mod.rs              # DB open/query API
│       └── schema.rs           # CREATE TABLE statements as constants
├── data/
│   └── puuzel.db               # committed or generated at build time
└── tools/
    ├── generate_clues.py       # offline Claude Batches API pipeline
    └── filter_wordlist.py      # OpenTaal → crossword-ready word list
```

### Pattern 1: Slot-Based CSP Generator

**What:** Model the crossword grid as a set of "slots" (horizontal and vertical word positions). Select slots in order of most-constrained first (fewest remaining valid words). Assign a word, propagate constraints to crossing slots, backtrack on failure.

**When to use:** Always — this is the standard algorithm for crossword generation.

**Key data structures:**
- `Slot`: `(row, col, direction, length, Vec<(position_in_slot, constrained_letter)>)`
- Position-letter index: `HashMap<(usize, char), Vec<WordId>>` — maps "position 2 must be 'A'" to all words satisfying that constraint. Built once at startup from the filtered word list.
- For each slot, the valid word set = intersection of the position-letter sets for all already-constrained positions.

**Example (conceptual):**
```rust
// Source: Algorithm documented at neilagrawal.com/post/implementing-csp-crossword-generation/
struct Slot {
    row: usize,
    col: usize,
    direction: Direction,
    length: usize,
    constraints: Vec<(usize, char)>, // (index_in_word, required_char)
}

fn candidates_for_slot(slot: &Slot, index: &PositionLetterIndex) -> Vec<WordId> {
    if slot.constraints.is_empty() {
        return index.words_of_length(slot.length);
    }
    // Intersect candidate sets for each constraint
    slot.constraints.iter()
        .map(|(pos, ch)| index.lookup(slot.length, *pos, *ch))
        .fold(None, |acc: Option<HashSet<WordId>>, set| {
            Some(match acc {
                None => set.iter().cloned().collect(),
                Some(a) => a.intersection(&set.iter().cloned().collect()).cloned().collect(),
            })
        })
        .unwrap_or_default()
        .into_iter().collect()
}
```

### Pattern 2: Dutch/European Grid Shape Rules

**What:** Unlike American crosswords, Dutch/European grids have:
- No rotational symmetry requirement
- Unchecked letters (unches) are normal and expected — up to ~50% of letters may be in only one word
- Two-letter words permitted
- Higher black square density than American style (25-40% black vs. ~16% American)
- No rule requiring every word to have a minimum crossing count

**When to use:** These rules directly affect the generator's validity checks. Do NOT enforce American-style symmetry or minimum-crossing rules.

**Grid validity rules to implement:**
1. White squares form one connected region (flood-fill check, mandatory — GRID-02)
2. No isolated single white squares (a 1-cell "word" is not a word)
3. All word slots have length >= 2 (GRID-05)
4. No word slot exceeds 15 characters (D-09)
5. No 2x2 block of all-white squares (European convention to avoid unstructured blobs)
6. Black square density within the configured range for the difficulty

**What NOT to enforce:**
- No symmetry check
- No minimum unch count requirement
- No maximum unch count per word

### Pattern 3: IJ Normalization

**What:** Before any word is stored or used in the generator, normalize the IJ digraph. The OpenTaal wordlist uses standard UTF-8. The IJ ligature (U+0132 / U+0133) may or may not appear; the two-ASCII-character sequence "IJ" is more common.

**Strategy:**
```rust
// Normalize a Dutch word for grid use:
// - Replace Unicode IJ ligature (U+0132) with the two-char sequence "IJ"
// - Then tokenize: split "IJ" into a single IjToken
// Result: word "IJSBEER" → tokens [IJ, S, B, E, E, R] → grid length 6

#[derive(Debug, Clone, PartialEq)]
enum LetterToken {
    Single(char),
    IJ,  // Represents the Dutch IJ digraph — counts as ONE grid cell
}

fn tokenize_dutch_word(word: &str) -> Vec<LetterToken> {
    // Normalize U+0132 to "IJ" first
    let normalized = word.replace('\u{0132}', "IJ").replace('\u{0133}', "ij");
    let upper = normalized.to_uppercase();
    let chars: Vec<char> = upper.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == 'I' && chars[i + 1] == 'J' {
            tokens.push(LetterToken::IJ);
            i += 2;
        } else {
            tokens.push(LetterToken::Single(chars[i]));
            i += 1;
        }
    }
    tokens
}
```

**Grid length:** Always use `tokens.len()`, not `word.len()`.

**Database storage:** Store the normalized form as a TEXT column with "IJ" as the two-char sequence (not the Unicode ligature). Grid length stored as a separate `grid_length INTEGER` column.

### Pattern 4: Claude Batch Clue Generation Pipeline

**What:** An offline Python script that batch-generates and self-verifies clues using the Claude Message Batches API.

**Pipeline steps:**
1. Load filtered word list (output of filter_wordlist.py)
2. For each word, construct a generation request:
   - Generate 3 clues (easy, medium, hard) in Dutch
   - Each clue max 6 words (D-17)
   - No wordplay/cryptic (D-18)
   - Rate word commonness 1-5
3. Submit as a batch (up to 100,000 requests per batch)
4. Poll for completion (typically < 1 hour)
5. For each generated clue, run a verification request:
   - "Given this clue in Dutch, what is the answer? Answer with a single word."
   - If returned answer != target word → mark clue as failed verification, exclude from DB
6. Write verified clues + metadata to SQLite

**Model recommendation:** Claude Haiku 3.5 for batch generation (lowest cost: $0.40/$2.00 per MTok input/output at batch pricing). Haiku 4.5 if clue quality proves insufficient ($0.50/$2.50). Both confirmed to support the Batches API.

**Two-pass vs single-pass:** Do generation and verification in two separate batches (simpler error handling) rather than a single multi-turn batch.

### Pattern 5: SQLite Schema

```sql
-- Source: Designed for this project; standard SQLite patterns
CREATE TABLE words (
    id INTEGER PRIMARY KEY,
    word TEXT NOT NULL UNIQUE,           -- normalized form, IJ as two chars
    grid_length INTEGER NOT NULL,        -- token count (IJ = 1)
    commonness_score INTEGER NOT NULL,   -- 1 (rare) to 5 (everyday); LLM-rated
    is_proper_noun INTEGER NOT NULL DEFAULT 0,  -- 1 for place names, people
    is_archaic INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE clues (
    id INTEGER PRIMARY KEY,
    word_id INTEGER NOT NULL REFERENCES words(id),
    difficulty TEXT NOT NULL CHECK(difficulty IN ('easy', 'medium', 'hard')),
    clue_text TEXT NOT NULL,             -- Dutch, max 6 words
    verified INTEGER NOT NULL DEFAULT 0, -- 1 if self-verification passed
    thumbs_down INTEGER NOT NULL DEFAULT 0  -- user blacklist flag
);

-- Indexes for puzzle generator queries
CREATE INDEX idx_words_grid_length ON words(grid_length);
CREATE INDEX idx_words_commonness ON words(commonness_score);
CREATE INDEX idx_clues_word_difficulty ON clues(word_id, difficulty);
CREATE INDEX idx_clues_verified ON clues(verified);
```

**Query pattern for word selection:**
```sql
-- Easy puzzle: short words, common vocab, verified clues exist
SELECT w.id, w.word, w.grid_length
FROM words w
WHERE w.grid_length BETWEEN 3 AND 8
  AND w.commonness_score >= 4
  AND EXISTS (SELECT 1 FROM clues c WHERE c.word_id = w.id AND c.difficulty = 'easy' AND c.verified = 1 AND c.thumbs_down = 0)
ORDER BY RANDOM()
LIMIT 200;
```

### Anti-Patterns to Avoid

- **Enforcing American-style symmetry:** Dutch grids have no symmetry requirement. Adding a symmetry check is wrong for this domain.
- **Building a custom hash map for word lookup:** The position-letter index using `HashMap<(usize, char), Vec<WordId>>` is sufficient and simple. A full trie is over-engineering for this use case.
- **Generating the grid structure then trying to fill words:** Generate the grid structure (black/white placement) and fill words simultaneously during the CSP search. Separating these two steps makes backtracking much harder.
- **Storing the Unicode IJ ligature (U+0132):** Store "IJ" as two ASCII characters. The ligature has no keyboard equivalent and causes inconsistency in string comparisons.
- **Runtime Claude API calls:** All clue data must be pre-generated and bundled. The game binary must never make HTTP calls.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SQLite bindings | Custom C FFI to SQLite | `rusqlite = { version = "0.39", features = ["bundled"] }` | rusqlite bundles SQLite 3.51.3, handles Flatpak sandbox, type-safe query API |
| Word list tokenization for IJ | Custom parser | The `tokenize_dutch_word` pattern above (10 lines) | Simple enough to write once; no crate needed |
| Flood-fill connectivity | Graph library | Standard BFS/DFS in 20 lines of Rust | Grid is small (20x20 = 400 cells); no library needed |
| Database migrations | Custom migration runner | `rusqlite_migration` crate (0.24.x) | Handles schema versioning safely; critical if DB format changes between releases |
| HTTP client for Claude API | `reqwest` in the game binary | External Python script using `anthropic` Python SDK | The pipeline is a one-time offline build step; Python is faster to write and has the official SDK |

**Key insight:** The word/clue database pipeline is a build-time script, not game code. Python is the right tool for that pipeline. The game binary only reads the pre-built SQLite file.

---

## Common Pitfalls

### Pitfall 1: Generator Hangs on Tight Grids

**What goes wrong:** With high word density (hard difficulty, 25-30% black squares) and a finite word list, the CSP search exhausts all valid completions and backtracks indefinitely.

**Why it happens:** Hard grids have more crossing constraints. When words are long and the dictionary has few matching words for a specific intersection pattern, the search space explodes.

**How to avoid:**
- Set a hard timeout (e.g., 8 seconds) and restart with a different random seed if exceeded
- Use "most constrained variable" heuristic — fill the slot with the fewest remaining candidates first
- Keep a minimum candidate threshold: if any slot has 0 candidates, backtrack immediately without exploring further
- For hard grids, ensure the word database has at least 2,000+ words in the 8-15 character range

**Warning signs:** Generation time > 3 seconds in testing; frequent restarts needed

### Pitfall 2: IJ Double-Counting in Grid Length

**What goes wrong:** A word like "RIJWIEL" (bicycle, 7 chars) contains no IJ, but "RIJKS" has no IJ either. "IJsberg" does. If grid_length is computed from `str.len()` rather than `tokenize().len()`, "IJSBEER" appears to be 7 cells wide but actually occupies 6. The puzzle generator places it in a 7-cell slot and produces an invalid crossing.

**Why it happens:** Forgetting to tokenize before measuring. Easy mistake when porting filter logic from a language-agnostic word list processor.

**How to avoid:** Always compute grid_length from `tokenize_dutch_word(word).len()` and store it in the DB. Never recompute from raw string length at grid-generation time.

**Warning signs:** Words with IJ appearing to overlap incorrectly with crossing words in test grids

### Pitfall 3: OpenTaal Inflected Forms Bloating the Word List

**What goes wrong:** The OpenTaal `wordlist.txt` (~400K entries) contains inflected forms (plurals, conjugations). Including "huis", "huizen", "huisje", "huisjes", "huiselijke" etc. as separate crossword entries wastes DB space and creates clue duplication.

**Why it happens:** The main wordlist.txt combines approved base words and inflected forms without separation.

**How to avoid:** Use `elements/basiswoorden-gekeurd.txt` (~200K approved base words) as the primary source. The `elements/flexies-ongekeurd.txt` (~170K) file contains inflected forms — exclude it. This brings the working set to ~200K before further filtering (length, vulgarity).

**Warning signs:** Word count > 150K after filtering for length 2-15; duplicate clues for morphologically related words

### Pitfall 4: Clue Self-Verification False Positives

**What goes wrong:** The Claude model generates a clue, then when asked to answer it, produces a synonym or related word rather than the exact target word. The clue is marked as failed even though it's valid.

**Why it happens:** For easy direct-definition clues, the verification model may answer with a valid synonym. Dutch synonyms are especially common for everyday words.

**How to avoid:**
- In the verification prompt, explicitly ask: "Answer with a single Dutch word that is the most direct answer to this clue."
- Accept the clue as verified if the verification answer either (a) matches the target word exactly, or (b) is a known inflected form of the target word
- Maintain a "soft fail" category for review rather than hard-rejecting all non-exact matches
- Target 80%+ verification pass rate; regenerate the batch for words below 50%

**Warning signs:** > 30% of easy clues failing verification; hard clues passing at higher rates than easy

### Pitfall 5: Connected White Squares Check Performance

**What goes wrong:** Running a flood-fill on every backtracking step makes generation extremely slow.

**Why it happens:** The connectivity check is O(grid size) = O(400), which is fast per invocation, but calling it thousands of times per generation adds up.

**How to avoid:** Only run the connectivity check (1) after the initial black square seeding, and (2) as a final validation step before returning the completed grid. During word placement, trust the CSP constraints to maintain connectivity — only fall back to flood-fill if a suspicious state is detected.

**Warning signs:** Generation taking > 5 seconds on simple grids

### Pitfall 6: No-Isolation Check for Black Square Seeding

**What goes wrong:** Random black square placement can create isolated white cells (a single white cell surrounded by black squares on all 4 sides). These are invalid — they form "words" of length 0 or 1.

**Why it happens:** Pure random black square seeding without post-placement validation.

**How to avoid:** After seeding black squares, run a quick validation: for each white cell, check that it has at least one white neighbor in the horizontal or vertical direction. If isolated cells exist, remove the nearest black square and reseed.

---

## Code Examples

Verified patterns from official sources:

### rusqlite 0.39 — Open bundled DB and query

```rust
// Source: https://docs.rs/rusqlite/latest/rusqlite/
use rusqlite::{Connection, Result, params};

pub fn open_database(path: &std::path::Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // Optimize for read-heavy workload (word queries during puzzle generation)
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    Ok(conn)
}

pub fn words_for_length(conn: &Connection, length: usize, min_commonness: i32) -> Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT id, word FROM words WHERE grid_length = ?1 AND commonness_score >= ?2 ORDER BY RANDOM() LIMIT 500"
    )?;
    let rows = stmt.query_map(params![length as i64, min_commonness], |row| {
        Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}
```

### rand 0.10 — RNG for grid generation

```rust
// Source: https://docs.rs/rand/latest/rand/
use rand::seq::SliceRandom;
use rand::seq::IndexedRandom;

fn shuffle_candidates(candidates: &mut Vec<WordId>) {
    let mut rng = rand::rng();
    candidates.shuffle(&mut rng);
}

fn pick_random<T>(items: &[T]) -> Option<&T> {
    let mut rng = rand::rng();
    items.choose(&mut rng)
}
```

**Note:** `rand::rng()` is the 0.10 API. Do NOT use `rand::thread_rng()` (0.8 API, removed in 0.10).

### OpenTaal word filter (Python, build-time)

```python
# tools/filter_wordlist.py
# Source: OpenTaal README + project decisions D-07 through D-13
import re

VULGARITY_LIST = set()  # populate from a curated blocklist

def load_and_filter(path: str) -> list[str]:
    results = []
    with open(path, encoding='utf-8') as f:
        for line in f:
            word = line.strip()
            if not word:
                continue
            # Length filter (D-09, D-10) — use grid_length after IJ normalization
            grid_len = compute_grid_length(word)
            if grid_len < 2 or grid_len > 15:
                continue
            # No abbreviations (D-07): skip words with dots or all-caps short words
            if '.' in word:
                continue
            # Vulgarity filter (D-11)
            if word.lower() in VULGARITY_LIST:
                continue
            results.append(word)
    return results

def compute_grid_length(word: str) -> int:
    """IJ digraph counts as 1 cell."""
    # Normalize Unicode IJ ligature
    normalized = word.replace('\u0132', 'IJ').replace('\u0133', 'ij').upper()
    count = 0
    i = 0
    while i < len(normalized):
        if i + 1 < len(normalized) and normalized[i] == 'I' and normalized[i+1] == 'J':
            count += 1
            i += 2
        else:
            count += 1
            i += 1
    return count
```

### Claude Batches API — clue generation request

```python
# tools/generate_clues.py
# Source: https://platform.claude.com/docs/en/build-with-claude/batch-processing
import anthropic

client = anthropic.Anthropic()

def make_generation_request(word: str, custom_id: str) -> dict:
    return {
        "custom_id": custom_id,
        "params": {
            "model": "claude-haiku-4-5",
            "max_tokens": 300,
            "messages": [{
                "role": "user",
                "content": f"""Genereer drie cryptogramloze aanwijzingen in het Nederlands voor het woord "{word}".
Regels:
- Elke aanwijzing is een directe definitie (geen woordspeling of cryptisch)
- Maximaal 6 woorden per aanwijzing
- Stijl: makkelijk = directe omschrijving, middel = feitelijk maar meer nadenken vereist, moeilijk = indirect of vraagt specifieke kennis
- Geef ook een gewoonheidsscore voor dit woord: 1=zeldzaam, 5=alledaags

Antwoord uitsluitend in dit JSON-formaat:
{{"easy": "...", "medium": "...", "hard": "...", "commonness": 3}}"""
            }]
        }
    }

def make_verification_request(word: str, clue: str, custom_id: str) -> dict:
    return {
        "custom_id": custom_id,
        "params": {
            "model": "claude-haiku-4-5",
            "max_tokens": 50,
            "messages": [{
                "role": "user",
                "content": f"""Los dit cryptogram op. Geef alleen het antwoord als één Nederlands woord.

Aanwijzing: {clue}"""
            }]
        }
    }
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Random word placement | CSP with forward-checking + MRV heuristic | ~2015 | 100x+ speed improvement on dense grids |
| rusqlite 0.32 (per CLAUDE.md) | rusqlite **0.39.0** (current) | 2025 | Version in CLAUDE.md is outdated; `bundled` feature still works identically |
| `rand::thread_rng()` (rand 0.8) | `rand::rng()` (rand 0.10) | rand 0.10.0 | Old API removed; will fail to compile if mixed |
| Unicode IJ ligature (U+0132) in word storage | Two-char ASCII "IJ" in storage | Project decision D-25 | Consistent display and comparison; ligature has no keyboard equivalent |

**Deprecated/outdated:**
- `rand::thread_rng()`: Removed in rand 0.10. Replaced by `rand::rng()`.
- `rand::gen()`: Replaced by `rand::random()` in rand 0.10.
- rusqlite 0.32 (listed in CLAUDE.md): Current version is 0.39.0. Update Cargo.toml.

---

## Open Questions

1. **Word list size after filtering**
   - What we know: OpenTaal basiswoorden-gekeurd.txt ~200K entries; filtering to length 2-15 chars probably yields ~150-180K entries; IJ normalization then further reduces by a small amount
   - What's unclear: How many of those 150K+ words will have passable AI-generated clues after verification? 50K verified words is more than enough; 10K might cause generator failures on hard grids
   - Recommendation: Run the filter script first (before clue generation) and count — if fewer than 20K unique grid-lengths 3-15 words remain, the filter is too aggressive

2. **Generator performance on 20x20 with real Dutch dictionary**
   - What we know: Academic research shows trie-based CSP generation for ~14x14 grids completes in < 1 second; Eyas's 10x10 grid blog achieved seconds with domain bisection; a Python CSP on a 20x20 achieved ~4 seconds
   - What's unclear: A 20x20 Rust implementation with a 50K+ Dutch word list has no published benchmark
   - Recommendation: Build a prototype generator in Week 1 and benchmark it on the actual filtered word list before committing to the algorithm. If it exceeds 5 seconds, add word-prefiltering by length into the slot selection step

3. **Vulgarity blocklist source for Dutch**
   - What we know: No open-licensed Dutch profanity list was found during research
   - What's unclear: Whether a small hand-curated list of ~200 words is sufficient, or whether a comprehensive source exists
   - Recommendation: Start with a hand-curated list of the most common Dutch vulgarities (~100-200 words). This is sufficient for v1 and avoids a dependency on an external source.

4. **Claude Haiku clue quality for Dutch**
   - What we know: Haiku 3.5 / 4.5 are the cheapest models supporting the Batches API; Claude 3.5 Sonnet produces higher quality outputs
   - What's unclear: Whether Haiku produces acceptable Dutch-language crossword clues or whether Sonnet is needed
   - Recommendation: Generate a test batch of 100 words with Haiku 4.5 first. If more than 30% fail verification or produce clearly nonsensical Dutch, upgrade to Sonnet 4 for the full run.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`cargo test`) |
| Config file | None — standard Rust test discovery |
| Quick run command | `cargo test --lib 2>&1 \| head -50` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GRID-01 | `generate_grid()` returns a `Grid` with ~20x20 dimensions | unit | `cargo test grid::tests::test_grid_dimensions` | Wave 0 |
| GRID-02 | All white cells in generated grid are connected (flood-fill) | unit | `cargo test grid::tests::test_white_squares_connected` | Wave 0 |
| GRID-03 | `tokenize_dutch_word("IJSBEER")` returns 6 tokens, first is `IJ` | unit | `cargo test grid::tests::test_ij_tokenization` | Wave 0 |
| GRID-04 | Generator does not fail on grids with unchecked letters | unit | `cargo test grid::tests::test_unchecked_letters_allowed` | Wave 0 |
| GRID-05 | Generator produces at least one 2-letter word slot | unit | `cargo test grid::tests::test_two_letter_words_present` | Wave 0 |
| GRID-06 | Easy grid has more black squares than hard grid | unit | `cargo test grid::tests::test_density_varies_by_difficulty` | Wave 0 |
| GRID-07 | Average word length is shorter in easy vs hard grid | unit | `cargo test grid::tests::test_word_length_by_difficulty` | Wave 0 |
| GRID-08 | Word query returns only words with commonness_score >= threshold | unit | `cargo test db::tests::test_word_commonness_filter` | Wave 0 |
| DATA-01 | Filter pipeline produces words of grid_length 2-15 with no abbreviations | unit (Python) | `python -m pytest tools/tests/test_filter.py` | Wave 0 |
| DATA-02 | Generated clues JSON has easy/medium/hard keys for each word | integration (manual) | Run test batch of 10 words, inspect output | Manual |
| DATA-03 | No clue contains "?" or wordplay markers | unit | `cargo test db::tests::test_clue_no_cryptic` | Wave 0 |
| DATA-04 | SQLite file opens without error, all tables present | unit | `cargo test db::tests::test_db_schema` | Wave 0 |
| DATA-05 | commonness_score in range 1-5 for all words | unit | `cargo test db::tests::test_commonness_range` | Wave 0 |
| DATA-06 | Verified clues have verified=1 in DB; failed clues excluded | unit | `cargo test db::tests::test_clue_verification_flag` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/grid/mod.rs` + `src/grid/generator.rs` — covers GRID-01 through GRID-07
- [ ] `src/db/mod.rs` + `src/db/schema.rs` — covers DATA-04 through DATA-06
- [ ] `src/grid/tests.rs` — all GRID-* unit tests
- [ ] `src/db/tests.rs` — all DATA-* unit tests
- [ ] `tools/tests/test_filter.py` — DATA-01 validation
- [ ] `Cargo.toml` — with rusqlite 0.39, rand 0.10, serde, log/env_logger

---

## Sources

### Primary (HIGH confidence)
- `cargo search rusqlite` — 0.39.0 confirmed on 2026-03-21
- `cargo search rand` — 0.10.0 confirmed on 2026-03-21
- `cargo search macroquad` — 0.4.14 confirmed on 2026-03-21
- https://docs.rs/rusqlite/latest/rusqlite/ — `bundled` feature, Connection API
- https://docs.rs/rand/latest/rand/ — `rng()`, `shuffle()`, `choose()` API in 0.10
- https://platform.claude.com/docs/en/build-with-claude/batch-processing — Batches API limits (100K requests, 50% discount, < 1 hour typical)
- https://github.com/OpenTaal/opentaal-wordlist — file structure verified: basiswoorden-gekeurd.txt ~200K, flexies-ongekeurd.txt ~170K

### Secondary (MEDIUM confidence)
- https://neilagrawal.com/post/implementing-csp-crossword-generation/ — CSP algorithm with position-letter index; 80-520ms on 8x14 grids (Python); Rust will be faster
- https://blog.eyas.sh/2025/12/algorithmic-crosswords/ — Domain bisection approach; 10x10 in seconds; constraint propagation cascade analysis
- https://www.baeldung.com/cs/generate-crossword-puzzle — Backtracking search overview; confirms 20x20 at ~4 seconds in Python (Rust estimate: < 1 second)
- https://www.mdpi.com/1999-4893/15/1/22 — Trie-based parallel crossword generation; confirms trie advantage for large dictionaries

### Tertiary (LOW confidence — needs validation)
- WebSearch results on Dutch crossword grid conventions — confirmed via Wikipedia crossword article that European style allows unchecked letters and two-letter words; Dutch-specific sourcing was not found
- Dutch vulgarity blocklist — no authoritative open-licensed source found; hand-curation recommended

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against crates.io registry on 2026-03-21
- Architecture (grid generator): HIGH — CSP with backtracking is well-established for crossword generation, multiple implementations documented
- Architecture (database schema): HIGH — rusqlite patterns are standard; schema design follows project decisions directly
- Architecture (IJ handling): HIGH — decision D-24 through D-29 are clear; tokenization pattern is straightforward
- Pitfalls: MEDIUM — performance pitfalls are inferred from algorithm analysis and community reports; 20x20 Rust benchmark not yet measured
- Clue pipeline: MEDIUM — Claude Batches API is well-documented; Dutch clue quality from Haiku is untested

**Research date:** 2026-03-21
**Valid until:** 2026-04-21 (stable stack); recheck rusqlite version if > 30 days pass
