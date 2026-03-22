use std::collections::HashMap;

use crate::db;
use crate::game::numbering::assign_clue_numbers;
use crate::grid::generator::FilledGrid;
use crate::grid::types::{Cell, Difficulty, Direction, Grid, LetterToken, Slot};

/// A single clue entry in the puzzle's clue list.
pub struct ClueEntry {
    pub number: u32,
    pub text: String,
    pub slot: Slot,
    pub word_id: i64,
}

/// The full state of an in-progress puzzle.
pub struct PuzzleState {
    /// The answer grid (filled with correct letters)
    pub grid: Grid,
    /// The player's entries — None means the cell is empty
    pub user_grid: Vec<Vec<Option<LetterToken>>>,
    /// Across clues, sorted by number
    pub across_clues: Vec<ClueEntry>,
    /// Down clues, sorted by number
    pub down_clues: Vec<ClueEntry>,
    /// Currently selected cell (row, col), if any
    pub selected_cell: Option<(usize, usize)>,
    /// Current typing direction
    pub selected_direction: Direction,
    /// Mapping from (row, col) to clue number for cells that start a clue
    pub clue_numbers: HashMap<(usize, usize), u32>,
    /// The difficulty level this puzzle was generated at
    pub difficulty: Difficulty,
    /// Set of (row, col) cells that are wrong (populated by check button)
    pub error_cells: Vec<(usize, usize)>,
}

impl PuzzleState {
    /// Construct a PuzzleState from a FilledGrid and a database connection.
    ///
    /// Looks up clue text for each word in the filled grid, assigns clue numbers,
    /// and builds the across/down clue lists.
    pub fn from_filled_grid(
        filled: FilledGrid,
        conn: &rusqlite::Connection,
    ) -> Result<Self, String> {
        let difficulty_str = match filled.difficulty {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
        };

        let width = filled.grid.width;
        let height = filled.grid.height;
        let user_grid = vec![vec![None; width]; height];

        let clue_numbers = assign_clue_numbers(&filled.grid);

        let mut across_clues: Vec<ClueEntry> = Vec::new();
        let mut down_clues: Vec<ClueEntry> = Vec::new();

        for (slot, word_id) in &filled.slot_words {
            let number = match clue_numbers.get(&(slot.row, slot.col)) {
                Some(&n) => n,
                None => {
                    // A slot that doesn't start at a numbered cell — skip it
                    // (shouldn't happen with valid grids, but handle gracefully)
                    continue;
                }
            };

            let clue_text = db::get_clue_for_word(conn, *word_id, difficulty_str)
                .map_err(|e| e.to_string())?
                .unwrap_or_else(|| "?".to_string());

            let entry = ClueEntry {
                number,
                text: clue_text,
                slot: slot.clone(),
                word_id: *word_id,
            };

            match slot.direction {
                Direction::Across => across_clues.push(entry),
                Direction::Down => down_clues.push(entry),
            }
        }

        across_clues.sort_by_key(|c| c.number);
        down_clues.sort_by_key(|c| c.number);

        Ok(PuzzleState {
            grid: filled.grid,
            user_grid,
            across_clues,
            down_clues,
            selected_cell: None,
            selected_direction: Direction::Across,
            clue_numbers,
            difficulty: filled.difficulty,
            error_cells: Vec::new(),
        })
    }

    /// Returns true if all white cells with answers have been correctly filled by the player.
    pub fn is_complete(&self) -> bool {
        for r in 0..self.grid.height {
            for c in 0..self.grid.width {
                if let Cell::White { letter: Some(ref answer) } = self.grid.cells[r][c] {
                    if self.user_grid[r][c] != Some(answer.clone()) {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Check all words against the answer grid. Marks entire words that contain
    /// any wrong letter, but excludes intersection cells where the crossing word
    /// is correct.
    pub fn check_errors(&mut self) {
        self.error_cells.clear();

        // Helper: check if a cell's user entry matches the answer
        let cell_correct = |r: usize, c: usize| -> bool {
            if let Cell::White { letter: Some(ref answer) } = self.grid.cells[r][c] {
                self.user_grid[r][c] == Some(answer.clone())
            } else {
                true // black cells are "correct"
            }
        };

        // Helper: get all cells in a slot
        let slot_cells = |slot: &Slot| -> Vec<(usize, usize)> {
            match slot.direction {
                Direction::Across => (slot.col..slot.col + slot.length)
                    .map(|c| (slot.row, c))
                    .collect(),
                Direction::Down => (slot.row..slot.row + slot.length)
                    .map(|r| (r, slot.col))
                    .collect(),
            }
        };

        // Find which words have errors
        let all_clues: Vec<&ClueEntry> = self.across_clues.iter()
            .chain(self.down_clues.iter())
            .collect();

        // For each cell, track if it belongs to a correct word in some direction
        let mut cell_in_correct_word: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
        let mut cells_to_mark: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

        // First pass: find correct and incorrect words
        for entry in &all_clues {
            let cells = slot_cells(&entry.slot);
            let word_has_error = cells.iter().any(|&(r, c)| !cell_correct(r, c));

            if word_has_error {
                for &(r, c) in &cells {
                    cells_to_mark.insert((r, c));
                }
            } else {
                for &(r, c) in &cells {
                    cell_in_correct_word.insert((r, c));
                }
            }
        }

        // Second pass: exclude cells that belong to a correct crossing word
        for &(r, c) in &cells_to_mark {
            if !cell_in_correct_word.contains(&(r, c)) {
                self.error_cells.push((r, c));
            }
        }
    }

    /// Returns the clue number of the active word (the word containing the selected cell
    /// in the current direction), if any.
    pub fn active_clue_number(&self) -> Option<u32> {
        let (sel_row, sel_col) = self.selected_cell?;

        // Walk backwards from the selected cell to find the start of the current word
        let word_start = match self.selected_direction {
            Direction::Across => {
                let mut col = sel_col;
                while col > 0 && matches!(self.grid.cells[sel_row][col - 1], Cell::White { .. }) {
                    col -= 1;
                }
                (sel_row, col)
            }
            Direction::Down => {
                let mut row = sel_row;
                while row > 0 && matches!(self.grid.cells[row - 1][sel_col], Cell::White { .. }) {
                    row -= 1;
                }
                (row, sel_col)
            }
        };

        self.clue_numbers.get(&word_start).copied()
    }

    /// Handle a cell click at (row, col).
    ///
    /// - If the cell is Black, the click is ignored.
    /// - If the clicked cell is already selected, toggle the direction (INTR-02, D-10).
    /// - Otherwise, select the cell and set direction to the word containing it
    ///   (prefer Across if both; if only one direction has a word, use that one).
    pub fn handle_cell_click(&mut self, row: usize, col: usize) {
        // Ignore clicks on black cells
        if matches!(self.grid.cells[row][col], Cell::Black) {
            return;
        }

        if self.selected_cell == Some((row, col)) {
            // Toggle direction
            self.selected_direction = match self.selected_direction {
                Direction::Across => Direction::Down,
                Direction::Down => Direction::Across,
            };
        } else {
            // Set new selection; prefer the direction with a valid word
            self.selected_cell = Some((row, col));

            let has_across = self.across_clues.iter().any(|e| {
                let s = &e.slot;
                s.row == row && s.col <= col && col < s.col + s.length
            });
            let has_down = self.down_clues.iter().any(|e| {
                let s = &e.slot;
                s.col == col && s.row <= row && row < s.row + s.length
            });

            // Prefer Across; fall back to Down; keep current if neither (shouldn't happen)
            if has_across {
                self.selected_direction = Direction::Across;
            } else if has_down {
                self.selected_direction = Direction::Down;
            }
        }
    }

    /// Set the letter in the selected cell and advance the cursor to the next white cell
    /// in the current direction (INTR-03, D-10).
    pub fn set_letter_and_advance(&mut self, ch: char) {
        let (row, col) = match self.selected_cell {
            Some(c) => c,
            None => return,
        };

        self.user_grid[row][col] = Some(LetterToken::Single(ch.to_ascii_uppercase()));
        // Clear error highlight on this cell when user types
        self.error_cells.retain(|&(r, c)| !(r == row && c == col));
        self.advance_cursor();
    }

    /// Advance the cursor to the next white cell in the current direction.
    /// Stops at the end of the word (does not wrap).
    fn advance_cursor(&mut self) {
        let (row, col) = match self.selected_cell {
            Some(c) => c,
            None => return,
        };

        match self.selected_direction {
            Direction::Across => {
                let mut next_col = col + 1;
                while next_col < self.grid.width {
                    if matches!(self.grid.cells[row][next_col], Cell::White { .. }) {
                        self.selected_cell = Some((row, next_col));
                        return;
                    }
                    next_col += 1;
                }
                // End of row — stay on current cell
            }
            Direction::Down => {
                let mut next_row = row + 1;
                while next_row < self.grid.height {
                    if matches!(self.grid.cells[next_row][col], Cell::White { .. }) {
                        self.selected_cell = Some((next_row, col));
                        return;
                    }
                    next_row += 1;
                }
                // End of column — stay on current cell
            }
        }
    }

    /// Handle the IJ digraph: if the current cell has a Single('I') and the answer is IJ,
    /// promote it to IJ and advance the cursor. Returns true if consumed.
    pub fn handle_ij_input(&mut self) -> bool {
        let (row, col) = match self.selected_cell {
            Some(c) => c,
            None => return false,
        };

        // Only promote if the answer at this cell is IJ
        let answer_is_ij = matches!(
            &self.grid.cells[row][col],
            Cell::White { letter: Some(LetterToken::IJ) }
        );

        // Only promote if the user has already typed 'I' in this cell
        let user_has_i = self.user_grid[row][col] == Some(LetterToken::Single('I'));

        if answer_is_ij && user_has_i {
            self.user_grid[row][col] = Some(LetterToken::IJ);
            self.advance_cursor();
            return true;
        }

        false
    }

    /// Handle backspace: clear the selected cell, or move back and clear the previous cell
    /// if the current cell is already empty (INTR-04, D-11).
    pub fn backspace(&mut self) {
        let (row, col) = match self.selected_cell {
            Some(c) => c,
            None => return,
        };

        if self.user_grid[row][col].is_some() {
            // Clear the current cell
            self.user_grid[row][col] = None;
        } else {
            // Move back one cell in the current direction and clear it
            match self.selected_direction {
                Direction::Across => {
                    if col > 0 {
                        // Find previous white cell
                        let mut prev_col = col;
                        while prev_col > 0 {
                            prev_col -= 1;
                            if matches!(self.grid.cells[row][prev_col], Cell::White { .. }) {
                                self.selected_cell = Some((row, prev_col));
                                self.user_grid[row][prev_col] = None;
                                return;
                            }
                        }
                    }
                }
                Direction::Down => {
                    if row > 0 {
                        let mut prev_row = row;
                        while prev_row > 0 {
                            prev_row -= 1;
                            if matches!(self.grid.cells[prev_row][col], Cell::White { .. }) {
                                self.selected_cell = Some((prev_row, col));
                                self.user_grid[prev_row][col] = None;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Move the cursor by `delta` cells in `direction` (D-12, INTR).
    /// Skips black cells. Clamps to grid bounds.
    pub fn move_cursor(&mut self, direction: Direction, delta: i32) {
        let (row, col) = match self.selected_cell {
            Some(c) => c,
            None => {
                // No selection: move to first white cell
                for r in 0..self.grid.height {
                    for c in 0..self.grid.width {
                        if matches!(self.grid.cells[r][c], Cell::White { .. }) {
                            self.selected_cell = Some((r, c));
                            self.selected_direction = direction;
                            return;
                        }
                    }
                }
                return;
            }
        };

        self.selected_direction = direction;

        match direction {
            Direction::Across => {
                let mut new_col = col as i32 + delta;
                new_col = new_col.max(0).min(self.grid.width as i32 - 1);
                // Skip black cells
                let step = if delta > 0 { 1i32 } else { -1 };
                let mut c = new_col;
                loop {
                    if c < 0 || c >= self.grid.width as i32 {
                        break;
                    }
                    if matches!(self.grid.cells[row][c as usize], Cell::White { .. }) {
                        self.selected_cell = Some((row, c as usize));
                        return;
                    }
                    c += step;
                }
                // Couldn't find a white cell in that direction; stay put
            }
            Direction::Down => {
                let mut new_row = row as i32 + delta;
                new_row = new_row.max(0).min(self.grid.height as i32 - 1);
                let step = if delta > 0 { 1i32 } else { -1 };
                let mut r = new_row;
                loop {
                    if r < 0 || r >= self.grid.height as i32 {
                        break;
                    }
                    if matches!(self.grid.cells[r as usize][col], Cell::White { .. }) {
                        self.selected_cell = Some((r as usize, col));
                        return;
                    }
                    r += step;
                }
            }
        }
    }

    /// Cycle through clues by `delta` (1 = next, -1 = previous), wrapping around (D-12).
    /// Selects the first cell of the new clue's slot.
    pub fn cycle_clue(&mut self, delta: i32) {
        // Build combined clue list: all across sorted by number, then all down sorted by number
        let combined: Vec<(Direction, u32, &crate::grid::types::Slot)> = self
            .across_clues
            .iter()
            .map(|e| (Direction::Across, e.number, &e.slot))
            .chain(
                self.down_clues
                    .iter()
                    .map(|e| (Direction::Down, e.number, &e.slot)),
            )
            .collect();

        if combined.is_empty() {
            return;
        }

        // Find current clue index
        let current_idx = self
            .selected_cell
            .and_then(|_| {
                let active_num = self.active_clue_number()?;
                combined
                    .iter()
                    .position(|(dir, num, _)| *dir == self.selected_direction && *num == active_num)
            })
            .unwrap_or(0);

        let n = combined.len() as i32;
        let new_idx = ((current_idx as i32 + delta).rem_euclid(n)) as usize;

        let (new_dir, _, slot) = combined[new_idx];
        self.selected_direction = new_dir;
        self.selected_cell = Some((slot.row, slot.col));
    }

    /// Select a clue by slot: set direction, find first empty cell in the slot (INTR-05, D-08).
    pub fn select_clue(&mut self, slot: &Slot) {
        self.selected_direction = slot.direction;

        // Find first empty cell in the slot
        let cells: Vec<(usize, usize)> = match slot.direction {
            Direction::Across => (slot.col..slot.col + slot.length)
                .map(|c| (slot.row, c))
                .collect(),
            Direction::Down => (slot.row..slot.row + slot.length)
                .map(|r| (r, slot.col))
                .collect(),
        };

        // Find first empty cell; if all filled, select the first cell
        let target = cells
            .iter()
            .find(|&&(r, c)| self.user_grid[r][c].is_none())
            .copied()
            .unwrap_or_else(|| cells[0]);

        self.selected_cell = Some(target);
    }
}

/// The top-level game state machine.
///
/// Note: cannot derive PartialEq or Clone because mpsc::Receiver does not implement them.
pub enum GameState {
    /// Showing the difficulty selection screen
    DifficultySelection,
    /// Puzzle is being generated in a background thread
    Generating {
        rx: std::sync::mpsc::Receiver<Result<PuzzleState, String>>,
    },
    /// A puzzle is loaded and the player is solving it
    Playing(PuzzleState),
    /// The player has completed the puzzle
    Congratulations(PuzzleState),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::types::{Cell, Grid, LetterToken};

    fn make_test_puzzle_state() -> PuzzleState {
        // Create a simple 3x3 grid: "CAT" across top row, all white
        let mut grid = Grid::new(3, 1);
        grid.cells[0][0] = Cell::White { letter: Some(LetterToken::Single('C')) };
        grid.cells[0][1] = Cell::White { letter: Some(LetterToken::Single('A')) };
        grid.cells[0][2] = Cell::White { letter: Some(LetterToken::Single('T')) };

        let clue_numbers = assign_clue_numbers(&grid);

        PuzzleState {
            grid,
            user_grid: vec![vec![None, None, None]],
            across_clues: vec![],
            down_clues: vec![],
            selected_cell: None,
            selected_direction: Direction::Across,
            clue_numbers,
            difficulty: Difficulty::Easy,
            error_cells: Vec::new(),
        }
    }

    #[test]
    fn test_is_complete_false_when_empty() {
        let state = make_test_puzzle_state();
        assert!(!state.is_complete(), "Puzzle with empty user_grid should not be complete");
    }

    #[test]
    fn test_is_complete_true_when_all_filled_correctly() {
        let mut state = make_test_puzzle_state();
        state.user_grid[0][0] = Some(LetterToken::Single('C'));
        state.user_grid[0][1] = Some(LetterToken::Single('A'));
        state.user_grid[0][2] = Some(LetterToken::Single('T'));
        assert!(state.is_complete(), "Puzzle should be complete when all cells correctly filled");
    }

    #[test]
    fn test_is_complete_false_when_wrong_letter() {
        let mut state = make_test_puzzle_state();
        state.user_grid[0][0] = Some(LetterToken::Single('C'));
        state.user_grid[0][1] = Some(LetterToken::Single('A'));
        state.user_grid[0][2] = Some(LetterToken::Single('X')); // Wrong!
        assert!(!state.is_complete(), "Puzzle should not be complete with wrong letter");
    }

    #[test]
    fn test_active_clue_number_no_selection() {
        let state = make_test_puzzle_state();
        assert_eq!(state.active_clue_number(), None, "No selection → no active clue");
    }

    #[test]
    fn test_active_clue_number_with_selection() {
        let mut state = make_test_puzzle_state();
        state.selected_cell = Some((0, 1)); // Middle of "CAT"
        state.selected_direction = Direction::Across;
        // Walking back from col 1: col 0 has White → word start is (0, 0)
        // (0, 0) should be numbered 1
        assert_eq!(state.active_clue_number(), Some(1), "Active clue for middle of CAT should be 1");
    }
}
