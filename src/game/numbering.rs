use std::collections::HashMap;
use crate::grid::types::{Cell, Grid};

/// Assign clue numbers to grid cells using standard crossword numbering rules.
///
/// Scans left-to-right, top-to-bottom. A white cell gets a number if it:
/// - Starts an Across word: at col==0 or left neighbor is Black, AND right neighbor exists and is White
/// - Starts a Down word: at row==0 or upper neighbor is Black, AND lower neighbor exists and is White
///
/// Words of length 1 do NOT get a number — the "next cell is White" check handles this.
/// Across and Down words sharing a start cell share the same number.
///
/// Returns a HashMap from (row, col) to clue number.
pub fn assign_clue_numbers(grid: &Grid) -> HashMap<(usize, usize), u32> {
    let mut numbers: HashMap<(usize, usize), u32> = HashMap::new();
    let mut next_number: u32 = 1;

    for row in 0..grid.height {
        for col in 0..grid.width {
            // Skip black cells
            if matches!(grid.cells[row][col], Cell::Black) {
                continue;
            }

            let starts_across = starts_across_word(grid, row, col);
            let starts_down = starts_down_word(grid, row, col);

            if starts_across || starts_down {
                numbers.insert((row, col), next_number);
                next_number += 1;
            }
        }
    }

    numbers
}

/// Returns true if the cell at (row, col) starts an Across word of length >= 2.
fn starts_across_word(grid: &Grid, row: usize, col: usize) -> bool {
    // Must be at left edge or have a black cell to the left
    let at_left_edge = col == 0;
    let left_is_black = col > 0 && matches!(grid.cells[row][col - 1], Cell::Black);

    if !at_left_edge && !left_is_black {
        return false;
    }

    // Right neighbor must exist and be White (word of length >= 2)
    col + 1 < grid.width && matches!(grid.cells[row][col + 1], Cell::White { .. })
}

/// Returns true if the cell at (row, col) starts a Down word of length >= 2.
fn starts_down_word(grid: &Grid, row: usize, col: usize) -> bool {
    // Must be at top edge or have a black cell above
    let at_top_edge = row == 0;
    let above_is_black = row > 0 && matches!(grid.cells[row - 1][col], Cell::Black);

    if !at_top_edge && !above_is_black {
        return false;
    }

    // Lower neighbor must exist and be White (word of length >= 2)
    row + 1 < grid.height && matches!(grid.cells[row + 1][col], Cell::White { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::types::{Cell, Grid};

    /// Build a grid from a string representation.
    /// '#' = Black, '.' = White (empty)
    fn grid_from_str(rows: &[&str]) -> Grid {
        let height = rows.len();
        let width = if height > 0 { rows[0].len() } else { 0 };
        let cells = rows
            .iter()
            .map(|row| {
                row.chars()
                    .map(|c| match c {
                        '#' => Cell::Black,
                        _ => Cell::White { letter: None },
                    })
                    .collect()
            })
            .collect();
        Grid { width, height, cells }
    }

    #[test]
    fn test_simple_3x3_numbering() {
        // Grid layout:
        // . . .
        // . # .
        // . . .
        //
        // Expected numbering:
        // Cell (0,0): starts Across (goes right) AND Down (goes below) → 1
        // Cell (0,1): starts Down (goes below is at (1,1) which is Black — no!), does NOT start across (not left edge or black left) → no number
        //   Actually (0,1): left neighbor (0,0) is White, so does NOT start Across.
        //   For Down: (0,1) is at top edge. Lower neighbor (1,1) is Black → does NOT start Down.
        //   → no number
        // Cell (0,2): starts Down only (top edge, (1,2) is White) → 2
        //   For Across: left (0,1) is White → does NOT start Across
        //   → 2
        // Cell (1,0): For Across: left edge, right (1,1) is Black → does NOT start Across.
        //   For Down: upper (0,0) is White → NOT at edge AND NOT black above → does NOT start Down.
        //   Wait — (1,0): upper neighbor (0,0) is White, so not starting Down.
        //   For Across: col==0 so left edge, but right (1,1) is Black → no.
        //   → no number
        // Cell (1,2): For Across: left (1,1) is Black AND right doesn't exist (col+1=3 >= width=3) → no.
        //   For Down: upper (0,2) is White → NOT starting Down.
        //   → no number
        // Cell (2,0): For Across: left edge, right (2,1) is White → starts Across → 3
        //   For Down: upper (1,0) is White → NOT starting Down.
        //   → 3
        // Cell (2,1): not left edge or black left → no Across.
        //   upper (1,1) is Black → starts Down? Lower (3,1) doesn't exist → NO.
        //   → no number
        // Cell (2,2): not left edge or black left → no Across.
        //   upper (1,2) is White → NOT starting Down.
        //   → no number
        //
        // Expected: (0,0)→1, (0,2)→2, (2,0)→3
        let grid = grid_from_str(&[
            "...",
            ".#.",
            "...",
        ]);

        let numbers = assign_clue_numbers(&grid);
        assert_eq!(numbers.get(&(0, 0)), Some(&1), "Top-left should be 1");
        assert_eq!(numbers.get(&(0, 2)), Some(&2), "Top-right should be 2");
        assert_eq!(numbers.get(&(2, 0)), Some(&3), "Bottom-left should be 3");
        assert_eq!(numbers.len(), 3, "Should have exactly 3 numbered cells");
    }

    #[test]
    fn test_single_cell_words_get_no_number() {
        // Grid:
        // . # .
        // # # #
        // . # .
        //
        // Each white cell is isolated — no word of length >= 2 can be formed.
        // → no numbered cells
        let grid = grid_from_str(&[
            ".#.",
            "###",
            ".#.",
        ]);

        let numbers = assign_clue_numbers(&grid);
        assert!(
            numbers.is_empty(),
            "Isolated white cells should produce no numbers, got: {:?}",
            numbers
        );
    }

    #[test]
    fn test_shared_number_for_across_and_down() {
        // Grid (5x1 row, all white):
        // . . . . .
        //
        // Only (0,0) should be numbered (starts Across, but no Down since row+1 doesn't exist)
        let grid = grid_from_str(&["....."]);
        let numbers = assign_clue_numbers(&grid);
        assert_eq!(numbers.get(&(0, 0)), Some(&1));
        assert_eq!(numbers.len(), 1);

        // Now a column: 5x1 column (5 rows, 1 col)
        // Only (0,0) should be numbered (starts Down, but no Across since col+1 doesn't exist)
        let grid_col = grid_from_str(&[".", ".", ".", ".", "."]);
        let numbers_col = assign_clue_numbers(&grid_col);
        assert_eq!(numbers_col.get(&(0, 0)), Some(&1));
        assert_eq!(numbers_col.len(), 1);
    }

    #[test]
    fn test_classic_plus_pattern() {
        // Grid:
        // # . #
        // . . .
        // # . #
        //
        // (0,1): top edge, lower (1,1) is White → starts Down. Left (0,0) is Black — wait no: (0,1) left is (0,0)=Black → starts Across? right (0,2) is Black → no. So only Down → 1
        // (1,0): left edge, right (1,1) is White → starts Across. upper (0,0) is Black → but row=1, row>0, upper (0,0) is Black → starts Down? lower (2,0) is Black → NO. So only Across → 2
        // (1,1): left (1,0) is White → not starts Across. upper (0,1) is White → not starts Down → no number
        // (1,2): left (1,1) is White → not starts Across. upper (0,2) is Black, lower (2,2) is Black → no Down → no number
        // (2,1): upper (1,1) is White → not starts Down. left (2,0) is Black → starts Across? right (2,2) is Black → NO → no number
        //
        // Expected: (0,1)→1, (1,0)→2
        let grid = grid_from_str(&[
            "#.#",
            "...",
            "#.#",
        ]);

        let numbers = assign_clue_numbers(&grid);
        assert_eq!(numbers.get(&(0, 1)), Some(&1), "(0,1) should be 1");
        assert_eq!(numbers.get(&(1, 0)), Some(&2), "(1,0) should be 2");
        assert_eq!(numbers.len(), 2, "Should have exactly 2 numbered cells");
    }
}
