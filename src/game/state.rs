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
