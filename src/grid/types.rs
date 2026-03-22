/// A single letter token in the Dutch crossword grid.
/// The IJ digraph is treated as a single cell in Dutch crosswords.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LetterToken {
    /// A standard single character
    Single(char),
    /// The Dutch IJ digraph, occupying a single grid cell
    IJ,
}

/// Direction of a word slot in the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Across,
    Down,
}

/// A cell in the crossword grid
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// A black (blocked) cell
    Black,
    /// A white cell, optionally containing a letter token
    White { letter: Option<LetterToken> },
}

/// A word slot in the grid — the location and shape of a word placement
#[derive(Debug, Clone)]
pub struct Slot {
    pub row: usize,
    pub col: usize,
    pub direction: Direction,
    pub length: usize,
}

/// Difficulty level for puzzle generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// Configuration for a given difficulty level
#[derive(Debug, Clone)]
pub struct DifficultyConfig {
    pub difficulty: Difficulty,
    pub black_square_ratio_min: f64,
    pub black_square_ratio_max: f64,
    pub min_word_length: usize,
    pub max_word_length: usize,
    /// Minimum commonness_score required for word selection
    pub min_commonness: i32,
}

impl DifficultyConfig {
    /// Easy: higher black square ratio (more open), common words, shorter max length
    pub fn easy() -> Self {
        DifficultyConfig {
            difficulty: Difficulty::Easy,
            black_square_ratio_min: 0.35,
            black_square_ratio_max: 0.40,
            min_word_length: 3,
            max_word_length: 8,
            min_commonness: 2,
        }
    }

    /// Medium: moderate black square ratio, moderately common words
    pub fn medium() -> Self {
        DifficultyConfig {
            difficulty: Difficulty::Medium,
            black_square_ratio_min: 0.30,
            black_square_ratio_max: 0.35,
            min_word_length: 3,
            max_word_length: 12,
            min_commonness: 3,
        }
    }

    /// Hard: lower black square ratio (denser grid), less common words, longer max length
    pub fn hard() -> Self {
        DifficultyConfig {
            difficulty: Difficulty::Hard,
            black_square_ratio_min: 0.25,
            black_square_ratio_max: 0.30,
            min_word_length: 3,
            max_word_length: 15,
            min_commonness: 1,
        }
    }
}

/// The crossword grid
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Cell>>,
}

impl Grid {
    /// Create a new grid with all cells initialized as white (empty)
    pub fn new(width: usize, height: usize) -> Self {
        let cells = (0..height)
            .map(|_| (0..width).map(|_| Cell::White { letter: None }).collect())
            .collect();
        Grid { width, height, cells }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_easy_ratios() {
        let cfg = DifficultyConfig::easy();
        assert_eq!(cfg.black_square_ratio_min, 0.35);
        assert_eq!(cfg.black_square_ratio_max, 0.40);
        assert_eq!(cfg.min_commonness, 2);
        assert_eq!(cfg.max_word_length, 8);
    }

    #[test]
    fn test_difficulty_medium_ratios() {
        let cfg = DifficultyConfig::medium();
        assert_eq!(cfg.black_square_ratio_min, 0.30);
        assert_eq!(cfg.black_square_ratio_max, 0.35);
        assert_eq!(cfg.min_commonness, 3);
        assert_eq!(cfg.max_word_length, 12);
    }

    #[test]
    fn test_difficulty_hard_ratios() {
        let cfg = DifficultyConfig::hard();
        assert_eq!(cfg.black_square_ratio_min, 0.25);
        assert_eq!(cfg.black_square_ratio_max, 0.30);
        assert_eq!(cfg.min_commonness, 1);
        assert_eq!(cfg.max_word_length, 15);
    }

    #[test]
    fn test_grid_new_all_white() {
        let grid = Grid::new(3, 2);
        assert_eq!(grid.width, 3);
        assert_eq!(grid.height, 2);
        assert_eq!(grid.cells.len(), 2);
        assert_eq!(grid.cells[0].len(), 3);
        for row in &grid.cells {
            for cell in row {
                assert_eq!(*cell, Cell::White { letter: None });
            }
        }
    }

    #[test]
    fn test_letter_token_ij_distinct_from_single() {
        let ij = LetterToken::IJ;
        let i = LetterToken::Single('I');
        let j = LetterToken::Single('J');
        assert_ne!(ij, i);
        assert_ne!(ij, j);
        assert_ne!(i, j);
    }
}
