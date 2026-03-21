use crate::grid::types::{Cell, Grid};

/// Checks whether all white cells in the grid form one connected region.
///
/// Uses BFS flood-fill from the first white cell found. Returns true if all
/// white cells are reachable, false if any white cell is isolated or in a
/// disconnected sub-region.
///
/// An empty grid (no white cells) returns true as a degenerate case.
pub fn is_connected(grid: &Grid) -> bool {
    let height = grid.height;
    let width = grid.width;

    // Find the first white cell
    let start = {
        let mut found = None;
        'outer: for r in 0..height {
            for c in 0..width {
                if matches!(grid.cells[r][c], Cell::White { .. }) {
                    found = Some((r, c));
                    break 'outer;
                }
            }
        }
        match found {
            None => return true, // no white cells — degenerate case
            Some(pos) => pos,
        }
    };

    // BFS flood fill
    let mut visited = vec![vec![false; width]; height];
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    visited[start.0][start.1] = true;

    while let Some((r, c)) = queue.pop_front() {
        // Check 4 orthogonal neighbors
        let neighbors: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];
        for (dr, dc) in neighbors {
            let nr = r as i32 + dr;
            let nc = c as i32 + dc;
            if nr < 0 || nr >= height as i32 || nc < 0 || nc >= width as i32 {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if !visited[nr][nc] && matches!(grid.cells[nr][nc], Cell::White { .. }) {
                visited[nr][nc] = true;
                queue.push_back((nr, nc));
            }
        }
    }

    // Verify all white cells were visited
    for r in 0..height {
        for c in 0..width {
            if matches!(grid.cells[r][c], Cell::White { .. }) && !visited[r][c] {
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::types::{Cell, Grid};

    fn all_white_grid(w: usize, h: usize) -> Grid {
        Grid::new(w, h)
    }

    fn grid_with_cells(cells: Vec<Vec<Cell>>) -> Grid {
        let height = cells.len();
        let width = if height > 0 { cells[0].len() } else { 0 };
        Grid { width, height, cells }
    }

    #[test]
    fn test_all_white_is_connected() {
        let grid = all_white_grid(20, 20);
        assert!(is_connected(&grid), "all-white 20x20 grid should be connected");
    }

    #[test]
    fn test_split_grid_is_disconnected() {
        // A 5x5 grid with a complete vertical black wall at col 2
        let mut grid = all_white_grid(5, 5);
        for r in 0..5 {
            grid.cells[r][2] = Cell::Black;
        }
        assert!(!is_connected(&grid), "grid split by vertical wall should be disconnected");
    }

    #[test]
    fn test_isolated_white_cell_disconnected() {
        // 3x3 grid, only white cell at (0,0) and (2,2) — rest black
        let mut cells = vec![vec![Cell::Black; 3]; 3];
        cells[0][0] = Cell::White { letter: None };
        cells[2][2] = Cell::White { letter: None };
        let grid = grid_with_cells(cells);
        assert!(!is_connected(&grid), "two isolated white cells should not be connected");
    }

    #[test]
    fn test_no_white_cells_returns_true() {
        // All-black grid — degenerate case should return true
        let cells = vec![vec![Cell::Black; 3]; 3];
        let grid = grid_with_cells(cells);
        assert!(is_connected(&grid), "all-black grid (no white cells) returns true");
    }

    #[test]
    fn test_single_white_cell_is_connected() {
        let mut cells = vec![vec![Cell::Black; 3]; 3];
        cells[1][1] = Cell::White { letter: None };
        let grid = grid_with_cells(cells);
        assert!(is_connected(&grid), "single white cell is trivially connected");
    }

    #[test]
    fn test_l_shaped_white_region_connected() {
        // 3x3 grid with L-shape: (0,0), (1,0), (2,0), (2,1), (2,2)
        let mut cells = vec![vec![Cell::Black; 3]; 3];
        cells[0][0] = Cell::White { letter: None };
        cells[1][0] = Cell::White { letter: None };
        cells[2][0] = Cell::White { letter: None };
        cells[2][1] = Cell::White { letter: None };
        cells[2][2] = Cell::White { letter: None };
        let grid = grid_with_cells(cells);
        assert!(is_connected(&grid), "L-shaped white region should be connected");
    }
}
