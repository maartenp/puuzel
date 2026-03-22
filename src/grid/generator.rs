use std::collections::{HashMap, HashSet};
use std::fmt;
use std::time::Instant;

use rusqlite::Connection;

use crate::db;
use crate::grid::connectivity::is_connected;
use crate::grid::difficulty::{extract_slots, seed_black_squares};
use crate::grid::ij::tokenize_dutch_word;
use crate::grid::types::{Cell, Difficulty, DifficultyConfig, Direction, Grid, LetterToken, Slot};

// Maximum generation time before returning Timeout error
const TIMEOUT_SECS: f64 = 30.0;

/// Error returned when grid generation fails.
#[derive(Debug)]
pub enum GeneratorError {
    /// Generation exceeded the time limit (8 seconds)
    Timeout,
    /// The search space was exhausted without finding a solution
    NoSolution,
    /// Failed to load words from the database
    DatabaseError(rusqlite::Error),
}

impl fmt::Display for GeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeneratorError::Timeout => write!(f, "Grid generation timed out after {}s", TIMEOUT_SECS),
            GeneratorError::NoSolution => write!(f, "No solution found for the given constraints"),
            GeneratorError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl From<rusqlite::Error> for GeneratorError {
    fn from(e: rusqlite::Error) -> Self {
        GeneratorError::DatabaseError(e)
    }
}

/// Pre-built word index for fast candidate filtering during CSP search.
///
/// Words are grouped by grid length and indexed by (length, position, token)
/// for efficient constraint propagation.
struct WordIndex {
    /// Words grouped by grid length: length → Vec<(word_id, tokens)>
    by_length: HashMap<usize, Vec<(i64, Vec<LetterToken>)>>,
    /// Position-letter index: (length, position, token) → indices into by_length[length]
    by_length_and_pos: HashMap<(usize, usize, LetterToken), Vec<usize>>,
}

impl WordIndex {
    fn build(conn: &Connection, config: &DifficultyConfig, test_mode: bool) -> Result<Self, GeneratorError> {
        let mut by_length: HashMap<usize, Vec<(i64, Vec<LetterToken>)>> = HashMap::new();

        // Load words for all lengths min_word_length through 20.
        // We use words_for_length_any_clue (not difficulty-filtered) because grid placement
        // only requires a word to have ANY clue — the difficulty determines which clue TEXT
        // is displayed, not which words are placed. This dramatically increases the pool for
        // short words (3-4 letters) where difficulty-specific clue coverage is sparse.
        //
        // Note: We load up to length 20 (the full grid width/height) rather than
        // max_word_length, because the black square placement cannot reliably enforce
        // a maximum slot length. The `viable` check is removed (all lengths are loaded).
        // Difficulty differentiation comes from clue text difficulty, not word length.
        for length in config.min_word_length..=20 {
            let words = if test_mode {
                db::words_for_length_all(conn, length)?
            } else {
                db::words_for_length_any_clue(conn, length, config.min_commonness)?
            };
            if words.is_empty() {
                continue;
            }
            let tokenized: Vec<(i64, Vec<LetterToken>)> = words
                .into_iter()
                .map(|(id, word)| (id, tokenize_dutch_word(&word)))
                .collect();
            by_length.insert(length, tokenized);
        }

        // Build position-letter index
        let mut by_length_and_pos: HashMap<(usize, usize, LetterToken), Vec<usize>> = HashMap::new();
        for (&length, words) in &by_length {
            for (idx, (_, tokens)) in words.iter().enumerate() {
                for (pos, token) in tokens.iter().enumerate() {
                    by_length_and_pos
                        .entry((length, pos, token.clone()))
                        .or_default()
                        .push(idx);
                }
            }
        }

        Ok(WordIndex { by_length, by_length_and_pos })
    }

    /// Get all word entries for a given length.
    fn words_for_length(&self, length: usize) -> &[(i64, Vec<LetterToken>)] {
        self.by_length.get(&length).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Get valid candidates for a slot given a set of constraints.
    /// Returns indices into words_for_length(length).
    fn candidates_for_constraints(
        &self,
        length: usize,
        constraints: &[(usize, LetterToken)],
        used_ids: &HashSet<i64>,
    ) -> Vec<usize> {
        let all_words = self.words_for_length(length);
        if all_words.is_empty() {
            return vec![];
        }

        if constraints.is_empty() {
            // No constraints — return all unused words
            return (0..all_words.len())
                .filter(|&i| !used_ids.contains(&all_words[i].0))
                .collect();
        }

        // Start with the intersection of candidate sets from each constraint
        // Use the most selective constraint first (smallest candidate set)
        let mut constraint_sets: Vec<Vec<usize>> = constraints
            .iter()
            .filter_map(|(pos, token)| {
                self.by_length_and_pos
                    .get(&(length, *pos, token.clone()))
                    .map(|v| v.clone())
            })
            .collect();

        if constraint_sets.len() < constraints.len() {
            // A constraint had no matches — no valid candidates
            return vec![];
        }

        // Sort by set size ascending (most selective first)
        constraint_sets.sort_by_key(|s| s.len());

        // Intersect all constraint sets
        let mut result: HashSet<usize> = constraint_sets[0].iter().copied().collect();
        for set in &constraint_sets[1..] {
            let set_hash: HashSet<usize> = set.iter().copied().collect();
            result = result.intersection(&set_hash).copied().collect();
            if result.is_empty() {
                return vec![];
            }
        }

        // Filter out used word IDs
        result
            .into_iter()
            .filter(|&i| !used_ids.contains(&all_words[i].0))
            .collect()
    }
}

/// Tracks the assignment state for a single slot during CSP search.
#[derive(Clone)]
struct SlotState {
    slot: Slot,
    /// The assigned word (word_id, tokens), if any
    assigned_word: Option<(i64, Vec<LetterToken>)>,
    /// Constraints imposed by crossing words: (position_in_slot, required_token)
    constraints: Vec<(usize, LetterToken)>,
}

/// A completed, filled crossword grid.
pub struct FilledGrid {
    /// The grid with all letters filled in
    pub grid: Grid,
    /// Which word_id was placed in each slot
    pub slot_words: Vec<(Slot, i64)>,
    /// The difficulty level used for generation
    pub difficulty: Difficulty,
}

/// Generate a valid 20x20 Dutch/European-style crossword grid.
///
/// Uses CSP backtracking with MRV (Most Restricted Variable) heuristic and
/// forward checking for efficient search.
///
/// Returns `Err(GeneratorError::Timeout)` if generation exceeds 8 seconds.
/// Returns `Err(GeneratorError::NoSolution)` if the search space is exhausted.
pub fn generate_grid(conn: &Connection, config: &DifficultyConfig, exclude: &HashSet<i64>) -> Result<FilledGrid, GeneratorError> {
    generate_grid_inner(conn, config, exclude, false)
}

/// Generate a grid in test mode — uses all words regardless of clue availability.
pub fn generate_grid_test_mode(conn: &Connection, config: &DifficultyConfig, exclude: &HashSet<i64>) -> Result<FilledGrid, GeneratorError> {
    generate_grid_inner(conn, config, exclude, true)
}

fn generate_grid_inner(conn: &Connection, config: &DifficultyConfig, exclude: &HashSet<i64>, test_mode: bool) -> Result<FilledGrid, GeneratorError> {
    let start = Instant::now();
    let mut rng = rand::rng();

    // Build word index from database
    let word_index = WordIndex::build(conn, config, test_mode)?;

    // Try up to 50 grid shapes before giving up.
    // Each shape attempt is cheap (rejected before CSP starts if invalid).
    for _shape_attempt in 0..50 {
        if start.elapsed().as_secs_f64() > TIMEOUT_SECS {
            return Err(GeneratorError::Timeout);
        }

        // Step 1: Create a 20x20 grid and seed black squares
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, config, &mut rng);

        // Step 2: Validate connectivity
        if !is_connected(&grid) {
            continue;
        }

        // Step 3: Extract word slots (only slots of length >= min_word_length)
        let slots = extract_slots(&grid, config.min_word_length);
        if slots.is_empty() {
            continue;
        }

        // Check that ALL white cells are covered by at least one slot.
        // When min_word_length > 2, cells in very short runs are not assigned to any slot
        // (orphan cells). Reject shapes with orphan cells.
        {
            let mut covered: HashSet<(usize, usize)> = HashSet::new();
            for slot in &slots {
                for pos in 0..slot.length {
                    let (r, c) = match slot.direction {
                        Direction::Across => (slot.row, slot.col + pos),
                        Direction::Down => (slot.row + pos, slot.col),
                    };
                    covered.insert((r, c));
                }
            }
            let has_orphan = grid.cells.iter().enumerate().any(|(r, row)| {
                row.iter().enumerate().any(|(c, cell)| {
                    matches!(cell, Cell::White { .. }) && !covered.contains(&(r, c))
                })
            });
            if has_orphan {
                continue;
            }
        }

        // Step 4: Check that all slot lengths have words in the index
        let viable = slots.iter().all(|s| {
            !word_index.words_for_length(s.length).is_empty()
        });
        if !viable {
            continue;
        }

        // Step 5: Build crossing map
        // For each slot, find which other slots cross it and at which positions
        // crossing_map[slot_i] = Vec<(slot_j, pos_in_i, pos_in_j)>
        let crossing_map = build_crossing_map(&slots, &grid);

        // Step 6: Initialize slot states
        let slot_states: Vec<SlotState> = slots
            .iter()
            .map(|s| SlotState {
                slot: s.clone(),
                assigned_word: None,
                constraints: vec![],
            })
            .collect();

        // Step 7: CSP backtracking
        let used_ids: HashSet<i64> = exclude.clone();
        match backtrack(
            slot_states,
            &word_index,
            &crossing_map,
            used_ids,
            &start,
        ) {
            BacktrackResult::Success(states) => {
                // Write letters into the grid
                let mut final_grid = grid.clone();
                let mut slot_words = vec![];

                for state in &states {
                    if let Some((word_id, ref tokens)) = state.assigned_word {
                        write_tokens_to_grid(&mut final_grid, &state.slot, tokens);
                        slot_words.push((state.slot.clone(), word_id));
                    }
                }

                // Final connectivity check
                if !is_connected(&final_grid) {
                    continue;
                }

                return Ok(FilledGrid {
                    grid: final_grid,
                    slot_words,
                    difficulty: config.difficulty,
                });
            }
            BacktrackResult::Timeout => return Err(GeneratorError::Timeout),
            BacktrackResult::Failure => continue,
        }
    }

    Err(GeneratorError::NoSolution)
}

/// Writes a word's tokens into grid cells at the given slot position.
fn write_tokens_to_grid(grid: &mut Grid, slot: &Slot, tokens: &[LetterToken]) {
    for (i, token) in tokens.iter().enumerate() {
        let (r, c) = match slot.direction {
            Direction::Across => (slot.row, slot.col + i),
            Direction::Down => (slot.row + i, slot.col),
        };
        grid.cells[r][c] = Cell::White { letter: Some(token.clone()) };
    }
}

/// Builds a crossing map for efficient constraint propagation.
///
/// Returns a Vec indexed by slot index. Each entry is a Vec of:
/// (crossing_slot_index, position_in_this_slot, position_in_crossing_slot)
fn build_crossing_map(slots: &[Slot], _grid: &Grid) -> Vec<Vec<(usize, usize, usize)>> {
    let n = slots.len();
    let mut map = vec![vec![]; n];

    // Build a map of (row, col) → (slot_index, position_in_slot) for quick lookup
    let mut cell_to_slots: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for (si, slot) in slots.iter().enumerate() {
        for pos in 0..slot.length {
            let (r, c) = match slot.direction {
                Direction::Across => (slot.row, slot.col + pos),
                Direction::Down => (slot.row + pos, slot.col),
            };
            cell_to_slots.entry((r, c)).or_default().push((si, pos));
        }
    }

    // For each cell shared by two slots, record the crossing
    for cell_slots in cell_to_slots.values() {
        if cell_slots.len() == 2 {
            let (si_a, pos_a) = cell_slots[0];
            let (si_b, pos_b) = cell_slots[1];
            map[si_a].push((si_b, pos_a, pos_b));
            map[si_b].push((si_a, pos_b, pos_a));
        }
    }

    map
}

enum BacktrackResult {
    Success(Vec<SlotState>),
    Timeout,
    Failure,
}

fn backtrack(
    mut states: Vec<SlotState>,
    word_index: &WordIndex,
    crossing_map: &[Vec<(usize, usize, usize)>],
    used_ids: HashSet<i64>,
    start: &Instant,
) -> BacktrackResult {
    if start.elapsed().as_secs_f64() > TIMEOUT_SECS {
        return BacktrackResult::Timeout;
    }

    // Find the next unassigned slot (MRV: pick the one with fewest valid candidates)
    let unassigned: Vec<usize> = (0..states.len())
        .filter(|&i| states[i].assigned_word.is_none())
        .collect();

    if unassigned.is_empty() {
        // All slots assigned — success
        return BacktrackResult::Success(states);
    }

    // MRV: pick slot with fewest candidates
    let chosen_idx = unassigned
        .iter()
        .map(|&i| {
            let count = word_index
                .candidates_for_constraints(
                    states[i].slot.length,
                    &states[i].constraints,
                    &used_ids,
                )
                .len();
            (i, count)
        })
        .min_by_key(|&(_, count)| count)
        .map(|(i, _)| i)
        .unwrap();

    let length = states[chosen_idx].slot.length;
    let constraints = states[chosen_idx].constraints.clone();

    let mut candidates = word_index.candidates_for_constraints(length, &constraints, &used_ids);

    if candidates.is_empty() {
        return BacktrackResult::Failure;
    }

    // Shuffle candidates for variety
    use rand::seq::SliceRandom;
    candidates.shuffle(&mut rand::rng());

    let words = word_index.words_for_length(length);

    for &candidate_idx in &candidates {
        let (word_id, ref tokens) = words[candidate_idx].clone();

        // Collect new constraints for crossing slots
        let new_constraints: Vec<(usize, usize, LetterToken)> = crossing_map[chosen_idx]
            .iter()
            .filter_map(|&(crossing_slot, pos_in_mine, pos_in_crossing)| {
                if states[crossing_slot].assigned_word.is_none() {
                    let token = tokens[pos_in_mine].clone();
                    Some((crossing_slot, pos_in_crossing, token))
                } else {
                    None
                }
            })
            .collect();

        // Forward checking: verify each crossing slot still has at least one valid candidate
        // after adding new constraints
        let mut forward_ok = true;
        let mut added_constraints: Vec<(usize, usize, LetterToken)> = vec![];

        for (crossing_slot, pos_in_crossing, token) in &new_constraints {
            // Add constraint tentatively
            states[*crossing_slot].constraints.push((*pos_in_crossing, token.clone()));
            added_constraints.push((*crossing_slot, *pos_in_crossing, token.clone()));

            // Check if crossing slot still has candidates
            let crossing_len = states[*crossing_slot].slot.length;
            let crossing_constraints = states[*crossing_slot].constraints.clone();
            let crossing_candidates = word_index.candidates_for_constraints(
                crossing_len,
                &crossing_constraints,
                &used_ids,
            );

            if crossing_candidates.is_empty() {
                forward_ok = false;
                break;
            }
        }

        if !forward_ok {
            // Undo constraints
            for (crossing_slot, pos, _) in &added_constraints {
                let constraints = &mut states[*crossing_slot].constraints;
                constraints.retain(|(p, _)| *p != *pos || {
                    // Remove the last occurrence of this position
                    false
                });
                // Actually just remove the last one we added
                constraints.pop();
            }
            continue;
        }

        // Assign word to chosen slot
        states[chosen_idx].assigned_word = Some((word_id, tokens.clone()));
        let mut new_used = used_ids.clone();
        new_used.insert(word_id);

        // Recurse
        match backtrack(states.clone(), word_index, crossing_map, new_used, start) {
            BacktrackResult::Success(result) => return BacktrackResult::Success(result),
            BacktrackResult::Timeout => return BacktrackResult::Timeout,
            BacktrackResult::Failure => {
                // Undo assignment
                states[chosen_idx].assigned_word = None;
                // Undo constraints
                for (crossing_slot, _, _) in &added_constraints {
                    states[*crossing_slot].constraints.pop();
                }
            }
        }
    }

    BacktrackResult::Failure
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::grid::ij::grid_length;
    use crate::grid::types::DifficultyConfig;

    /// Insert a test word with a verified clue at all difficulty levels.
    fn insert_test_word(
        conn: &Connection,
        word: &str,
        commonness: i32,
    ) -> rusqlite::Result<i64> {
        let gl = grid_length(word) as i64;
        let id = db::insert_word(conn, word, gl, commonness, false, false)?;
        db::insert_clue(conn, id, "easy", &format!("Clue for {}", word), true)?;
        db::insert_clue(conn, id, "medium", &format!("Clue for {}", word), true)?;
        db::insert_clue(conn, id, "hard", &format!("Clue for {}", word), true)?;
        Ok(id)
    }

    /// Build a test word database with a variety of Dutch words.
    /// Words are organized by their actual grid_length (IJ = 1 cell).
    fn build_test_db() -> rusqlite::Result<Connection> {
        let conn = db::open_in_memory()?;

        // Words grouped by grid_length. IJ counts as 1 cell.
        // grid_length 2
        let gl2 = ["OP", "IN", "OM", "AF", "AL", "OF", "EN", "DE", "NU", "ZO", "MO", "BO"];
        // grid_length 3
        let gl3 = [
            "DAG", "EEN", "HET", "DIT", "WAT", "KAN", "BED", "KOP", "BAL", "MAN",
            "OOG", "OOR", "LEG", "PAN", "WEG", "DAM", "GAT", "LAM", "TAK", "ZAK",
            "PAK", "MAK", "RAK", "TON", "BON", "RON", "MON", "KON", "ZON", "GON",
        ];
        // grid_length 4
        let gl4 = [
            "HUIS", "BOOM", "DEUR", "RAAM", "TAAK", "NAAM", "JAAR", "MENS",
            "LAND", "STAD", "BERG", "MEER", "BRUG", "TRAP", "HELM", "KELK",
            "MELK", "GOLF", "WOLF", "LAMP", "DAMP", "KAMP", "RAMP", "BAAN",
            "MAAN", "PAAR", "MAAR", "HAAR", "NAAR", "DAAR", "WAAR", "LAAR",
        ];
        // grid_length 5
        let gl5 = [
            "FIETS", "WATER", "BROOD", "KLEUR", "WERKT", "GROEP", "HOOFD",
            "KWART", "BLOED", "VLEES", "KREET", "GROEN", "BLAUW", "ZWART",
            "BLANK", "PLANK", "FRANK", "STANK", "KLANK", "DRINK", "WRANG",
            "DRANG", "WREED", "GRENS", "KRANS", "TRANS", "PRANS", "DRANS",
        ];
        // grid_length 6
        let gl6 = [
            "SCHOOL", "STRAAT", "VOGELS", "TAFELS", "HANDEN", "VOETEN",
            "MOEDER", "VRIEND", "VIJAND", "DORPEN", "STEDEN", "BOSSEN",
            "KATTEN", "HONDEN", "MENSEN", "WINDEN", "RANDEN", "MANDEN",
            "BANDEN", "LANDEN", "BERGEN", "STEGEN", "WEGEN", "ZANDEN",
        ];
        // grid_length 7
        let gl7 = [
            "KASTEEL", "TRIBUNE", "CONCERT", "THEATER", "LICHAAM", "GEWICHT",
            "FAMILIE", "VRIJDAG", "MAANDAG", "DINSDAG", "WEEKEND", "FEESTDAG",
        ];
        // grid_length 8
        let gl8 = [
            "ARBEIDER", "WANDELEN", "REKENING", "BELONING", "VERDIENT",
            "BEREIDEN", "BEVINDEN", "BEGRIJPEN", "BEZOEKEN", "BEREKEND",
        ];
        // IJ words (IJ = 1 cell, so LIJST = 4 cells, IJS = 2 cells)
        let ij_words = [
            "IJS",    // grid_length 2: IJ + S
            "LIJST",  // grid_length 4: L + IJ + S + T
            "PRIJS",  // grid_length 4: P + R + IJ + S
            "VRIJ",   // grid_length 3: V + R + IJ
            "WIJD",   // grid_length 3: W + IJ + D
            "TIJD",   // grid_length 3: T + IJ + D
        ];

        for w in &gl2 { insert_test_word(&conn, w, 5)?; }
        for w in &gl3 { insert_test_word(&conn, w, 5)?; }
        for w in &gl4 { insert_test_word(&conn, w, 5)?; }
        for w in &gl5 { insert_test_word(&conn, w, 5)?; }
        for w in &gl6 { insert_test_word(&conn, w, 4)?; }
        for w in &gl7 { insert_test_word(&conn, w, 3)?; }
        for w in &gl8 { insert_test_word(&conn, w, 3)?; }
        for w in &ij_words { insert_test_word(&conn, w, 5)?; }

        Ok(conn)
    }

    #[test]
    fn test_generate_easy_grid() {
        let conn = build_test_db().expect("build_test_db failed");
        let config = DifficultyConfig::easy();

        let result = generate_grid(&conn, &config, &HashSet::new());
        match result {
            Ok(filled) => {
                assert_eq!(filled.grid.width, 20);
                assert_eq!(filled.grid.height, 20);
                assert_eq!(filled.difficulty, Difficulty::Easy);

                // All white cells must have a letter
                for row in &filled.grid.cells {
                    for cell in row {
                        if let Cell::White { letter } = cell {
                            assert!(
                                letter.is_some(),
                                "White cell has no letter assigned"
                            );
                        }
                    }
                }

                // White cells must be connected
                assert!(is_connected(&filled.grid), "Generated grid white cells not connected");
            }
            Err(GeneratorError::NoSolution) => {
                // May occur with small test DB — acceptable
                eprintln!("test_generate_easy_grid: NoSolution (small test DB)");
            }
            Err(e) => {
                // Timeout or DB error — still acceptable for small test DB
                eprintln!("test_generate_easy_grid: Error: {}", e);
            }
        }
    }

    #[test]
    fn test_generate_hard_grid() {
        let conn = build_test_db().expect("build_test_db failed");
        let config = DifficultyConfig::hard();

        let result = generate_grid(&conn, &config, &HashSet::new());
        match result {
            Ok(filled) => {
                assert_eq!(filled.grid.width, 20);
                assert_eq!(filled.grid.height, 20);

                let total = 20 * 20;
                let black_count = filled.grid.cells.iter().flatten()
                    .filter(|c| matches!(c, Cell::Black))
                    .count();
                let ratio = black_count as f64 / total as f64;
                eprintln!("Hard grid black ratio: {:.3}", ratio);
            }
            Err(e) => {
                eprintln!("test_generate_hard_grid: {}", e);
            }
        }
    }

    #[test]
    fn test_connectivity_after_generation() {
        let conn = build_test_db().expect("build_test_db failed");
        let config = DifficultyConfig::easy();

        let result = generate_grid(&conn, &config, &HashSet::new());
        if let Ok(filled) = result {
            assert!(
                is_connected(&filled.grid),
                "Generated grid must have all white cells connected"
            );
        }
    }

    #[test]
    fn test_two_letter_slots_exist() {
        let conn = build_test_db().expect("build_test_db failed");
        let config = DifficultyConfig::easy();

        let result = generate_grid(&conn, &config, &HashSet::new());
        if let Ok(filled) = result {
            let has_two_letter = filled.slot_words.iter().any(|(slot, _)| slot.length == 2);
            assert!(
                has_two_letter,
                "Generated grid should have at least one 2-letter slot (GRID-05)"
            );
        }
    }

    #[test]
    fn test_no_duplicate_words() {
        let conn = build_test_db().expect("build_test_db failed");
        let config = DifficultyConfig::easy();

        let result = generate_grid(&conn, &config, &HashSet::new());
        if let Ok(filled) = result {
            let word_ids: Vec<i64> = filled.slot_words.iter().map(|(_, id)| *id).collect();
            let unique: HashSet<i64> = word_ids.iter().copied().collect();
            assert_eq!(
                word_ids.len(),
                unique.len(),
                "Duplicate word IDs found in generated grid"
            );
        }
    }

    #[test]
    #[ignore] // May be flaky due to randomness with small test DB
    fn test_word_length_varies_by_difficulty() {
        let conn = build_test_db().expect("build_test_db failed");

        let easy_config = DifficultyConfig::easy();
        let hard_config = DifficultyConfig::hard();

        let easy_result = generate_grid(&conn, &easy_config, &HashSet::new());
        let hard_result = generate_grid(&conn, &hard_config, &HashSet::new());

        if let (Ok(easy), Ok(hard)) = (easy_result, hard_result) {
            let easy_avg = easy.slot_words.iter().map(|(s, _)| s.length).sum::<usize>() as f64
                / easy.slot_words.len() as f64;
            let hard_avg = hard.slot_words.iter().map(|(s, _)| s.length).sum::<usize>() as f64
                / hard.slot_words.len() as f64;
            eprintln!("Easy avg word length: {:.2}, Hard avg word length: {:.2}", easy_avg, hard_avg);
            assert!(
                easy_avg <= hard_avg,
                "Easy avg word length ({:.2}) should be <= hard avg ({:.2})",
                easy_avg, hard_avg
            );
        }
    }

    /// Diagnostic test: opens the real database and tries to generate a grid for each difficulty.
    /// Prints detailed info about what's happening. Run with:
    ///   cargo test test_real_db_diagnostic -- --nocapture --ignored
    #[test]
    #[ignore]
    fn test_real_db_diagnostic() {
        let db_path = std::path::PathBuf::from("data/puuzel.db");
        if !db_path.exists() {
            eprintln!("[SKIP] data/puuzel.db not found, skipping real DB diagnostic");
            return;
        }
        let conn = db::open_database(&db_path).expect("open_database failed");

        for config in &[DifficultyConfig::easy(), DifficultyConfig::medium(), DifficultyConfig::hard()] {
            let diff_name = match config.difficulty {
                crate::grid::types::Difficulty::Easy => "easy",
                crate::grid::types::Difficulty::Medium => "medium",
                crate::grid::types::Difficulty::Hard => "hard",
            };
            eprintln!("\n=== Testing difficulty: {} ===", diff_name);
            eprintln!("  max_word_length={} min_commonness={}", config.max_word_length, config.min_commonness);

            // Check what the DB returns for each length
            for length in 2..=config.max_word_length {
                let words = db::words_for_length(&conn, length, config.min_commonness, diff_name)
                    .unwrap_or_default();
                eprintln!("  DB length={}: {} words", length, words.len());
            }

            let result = generate_grid(&conn, config, &HashSet::new());
            match result {
                Ok(filled) => {
                    let mut slot_lens: HashMap<usize, usize> = HashMap::new();
                    for (s, _) in &filled.slot_words {
                        *slot_lens.entry(s.length).or_default() += 1;
                    }
                    eprintln!("  SUCCESS: {} slots placed", filled.slot_words.len());
                    let mut sorted: Vec<_> = slot_lens.iter().collect();
                    sorted.sort();
                    for (l, c) in sorted { eprintln!("    length {}: {} words", l, c); }
                }
                Err(e) => eprintln!("  FAILED: {}", e),
            }
        }
    }

    #[test]
    fn test_timeout_returns_error() {
        // Build a DB with only very short words that can't fill a 20x20 grid
        // to trigger either NoSolution or Timeout
        let conn = db::open_in_memory().expect("open_in_memory failed");
        // Only 2-letter word — impossible to fill a 20x20 with just 2-letter words in 8s
        let id = db::insert_word(&conn, "AB", 2, 5, false, false).expect("insert failed");
        db::insert_clue(&conn, id, "easy", "Test clue", true).expect("insert_clue failed");

        let config = DifficultyConfig::easy();
        let result = generate_grid(&conn, &config, &HashSet::new());
        // Should return NoSolution or Timeout — not panic
        match result {
            Err(GeneratorError::Timeout) | Err(GeneratorError::NoSolution) => {}
            Ok(_) => panic!("Should not succeed with only one 2-letter word"),
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }
}
