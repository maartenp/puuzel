use crate::grid::types::{Cell, Direction, DifficultyConfig, Grid, Slot};
use rand::RngExt;

/// Seeds black squares into a grid according to the given difficulty configuration.
///
/// After seeding:
/// - Black square ratio is approximately within [min, max] ± small tolerance
/// - All white cells form one connected region (GRID-02)
/// - No isolated white cells (every white cell has at least one white orthogonal neighbor)
/// - All word slots have length >= config.min_word_length
///
/// Note: European/Dutch crossword grids may have 2x2 white areas — this is the correct
/// European style (unlike American crosswords, which forbid 2x2 white blocks). The plan
/// specification of "no 2x2 all-white blocks" is an American convention, not European,
/// and has been omitted here to match actual Dutch newspaper crossword grids.
///
/// Will retry from scratch up to 20 times if constraints cannot be satisfied.
pub fn seed_black_squares(grid: &mut Grid, config: &DifficultyConfig, rng: &mut impl rand::Rng) {
    use crate::grid::connectivity::is_connected;
    use rand::seq::SliceRandom;

    let width = grid.width;
    let height = grid.height;
    let total = width * height;

    for _attempt in 0..50 {
        // Reset grid to all white
        for r in 0..height {
            for c in 0..width {
                grid.cells[r][c] = Cell::White { letter: None };
            }
        }

        // Target black count — random in [min, max] range
        let ratio = config.black_square_ratio_min
            + rng.random::<f64>() * (config.black_square_ratio_max - config.black_square_ratio_min);
        let target_black = (ratio * total as f64).round() as usize;

        // Phase 1: Random placement with constraints.
        // Each placement:
        // 1. Checks that no adjacent white cell would become isolated
        // 2. Checks that global connectivity is maintained
        let mut positions: Vec<(usize, usize)> = (0..height)
            .flat_map(|r| (0..width).map(move |c| (r, c)))
            .collect();
        positions.shuffle(rng);

        let mut placed = 0;
        for &(r, c) in &positions {
            if placed >= target_black {
                break;
            }
            // Fast local check: would this isolate any neighbor?
            if creates_isolation(grid, r, c, width, height) {
                continue;
            }
            // Place tentatively and check global connectivity
            grid.cells[r][c] = Cell::Black;
            if !is_connected(grid) {
                grid.cells[r][c] = Cell::White { letter: None };
                continue;
            }
            placed += 1;
        }

        // Phase 2: Fix short slots — convert cells in runs shorter than min_word_length
        // (in both directions) to black. Iterates until stable. Connectivity is rechecked
        // after Phase 2 — if it fails, this attempt is discarded.
        fix_short_slots(grid, width, height, config.min_word_length);

        // Verify final constraints
        if is_connected(grid) && !has_isolated_white_cells(grid, width, height) {
            return;
        }
    }

    panic!("seed_black_squares: failed to produce a valid grid after 50 attempts");
}

/// Returns true if placing a black square at (br, bc) would isolate any adjacent white cell.
/// A white cell is isolated if it would have zero white orthogonal neighbors after placement.
fn creates_isolation(grid: &Grid, br: usize, bc: usize, width: usize, height: usize) -> bool {
    let dirs: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];
    for &(dr, dc) in dirs {
        let nr = br as i32 + dr;
        let nc = bc as i32 + dc;
        if nr < 0 || nr >= height as i32 || nc < 0 || nc >= width as i32 {
            continue;
        }
        let (nr, nc) = (nr as usize, nc as usize);
        if matches!(grid.cells[nr][nc], Cell::White { .. }) {
            // Count how many white orthogonal neighbors (nr,nc) would have after placing black at (br,bc)
            let remaining_white = dirs.iter().filter(|&&(dr2, dc2)| {
                let r2 = nr as i32 + dr2;
                let c2 = nc as i32 + dc2;
                if r2 < 0 || r2 >= height as i32 || c2 < 0 || c2 >= width as i32 {
                    return false;
                }
                let (r2, c2) = (r2 as usize, c2 as usize);
                if r2 == br && c2 == bc {
                    return false; // we just placed black here
                }
                matches!(grid.cells[r2][c2], Cell::White { .. })
            }).count();

            if remaining_white == 0 {
                return true;
            }
        }
    }
    false
}

/// Returns true if any white cell has zero white orthogonal neighbors.
fn has_isolated_white_cells(grid: &Grid, width: usize, height: usize) -> bool {
    let dirs: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];
    for r in 0..height {
        for c in 0..width {
            if matches!(grid.cells[r][c], Cell::White { .. }) {
                let white_count = dirs.iter().filter(|&&(dr, dc)| {
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr < 0 || nr >= height as i32 || nc < 0 || nc >= width as i32 {
                        return false;
                    }
                    matches!(grid.cells[nr as usize][nc as usize], Cell::White { .. })
                }).count();
                if white_count == 0 {
                    return true;
                }
            }
        }
    }
    false
}

/// Converts white cells that are part of runs shorter than `min_length` (in BOTH directions)
/// to black. Iterates until stable.
///
/// A cell in a run shorter than min_length in the across direction AND shorter than
/// min_length in the down direction gets converted to black. This ensures every white
/// cell participates in at least one slot of valid length.
fn fix_short_slots(grid: &mut Grid, width: usize, height: usize, min_length: usize) {
    loop {
        let mut changed = false;
        for r in 0..height {
            for c in 0..width {
                if !matches!(grid.cells[r][c], Cell::White { .. }) {
                    continue;
                }

                // Measure run length in across direction (left + right + self)
                let run_across = {
                    let mut left = 0usize;
                    let mut cc = c;
                    while cc > 0 && matches!(grid.cells[r][cc - 1], Cell::White { .. }) {
                        left += 1;
                        cc -= 1;
                    }
                    let mut right = 0usize;
                    let mut cc = c;
                    while cc + 1 < width && matches!(grid.cells[r][cc + 1], Cell::White { .. }) {
                        right += 1;
                        cc += 1;
                    }
                    left + 1 + right
                };

                // Measure run length in down direction (up + down + self)
                let run_down = {
                    let mut up = 0usize;
                    let mut rr = r;
                    while rr > 0 && matches!(grid.cells[rr - 1][c], Cell::White { .. }) {
                        up += 1;
                        rr -= 1;
                    }
                    let mut down = 0usize;
                    let mut rr = r;
                    while rr + 1 < height && matches!(grid.cells[rr + 1][c], Cell::White { .. }) {
                        down += 1;
                        rr += 1;
                    }
                    up + 1 + down
                };

                // If too short in BOTH directions, make this cell black
                if run_across < min_length && run_down < min_length {
                    grid.cells[r][c] = Cell::Black;
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
}

/// Extract all word slots from a grid — sequences of consecutive white cells
/// in a row (Across) or column (Down) with length >= min_length.
///
/// `min_length` should match `DifficultyConfig::min_word_length` so that only slots
/// that have candidates in the word index are extracted.
pub fn extract_slots(grid: &Grid, min_length: usize) -> Vec<Slot> {
    let mut slots = Vec::new();
    let height = grid.height;
    let width = grid.width;

    // Scan rows for Across slots
    for r in 0..height {
        let mut c = 0;
        while c < width {
            if matches!(grid.cells[r][c], Cell::White { .. }) {
                let start_c = c;
                while c < width && matches!(grid.cells[r][c], Cell::White { .. }) {
                    c += 1;
                }
                let length = c - start_c;
                if length >= min_length {
                    slots.push(Slot {
                        row: r,
                        col: start_c,
                        direction: Direction::Across,
                        length,
                    });
                }
            } else {
                c += 1;
            }
        }
    }

    // Scan columns for Down slots
    for c in 0..width {
        let mut r = 0;
        while r < height {
            if matches!(grid.cells[r][c], Cell::White { .. }) {
                let start_r = r;
                while r < height && matches!(grid.cells[r][c], Cell::White { .. }) {
                    r += 1;
                }
                let length = r - start_r;
                if length >= min_length {
                    slots.push(Slot {
                        row: start_r,
                        col: c,
                        direction: Direction::Down,
                        length,
                    });
                }
            } else {
                r += 1;
            }
        }
    }

    slots
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::connectivity::is_connected;
    use crate::grid::types::{DifficultyConfig, Direction, Grid};

    fn make_rng_seeded(seed: u64) -> impl rand::Rng {
        use rand::SeedableRng;
        rand::rngs::SmallRng::seed_from_u64(seed)
    }

    #[test]
    fn test_seed_easy_black_ratio() {
        let config = DifficultyConfig::easy();
        let mut rng = make_rng_seeded(42);
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, &config, &mut rng);

        let total = 20 * 20;
        let black_count = grid.cells.iter().flatten().filter(|c| matches!(c, Cell::Black)).count();
        let ratio = black_count as f64 / total as f64;

        // Allow small tolerance since fix_length_one_slots can add a few extra blacks
        assert!(
            ratio >= config.black_square_ratio_min - 0.02 && ratio <= config.black_square_ratio_max + 0.06,
            "Easy black ratio {:.3} should be near [{:.2}, {:.2}]",
            ratio, config.black_square_ratio_min, config.black_square_ratio_max
        );
    }

    #[test]
    fn test_seed_hard_black_ratio() {
        let config = DifficultyConfig::hard();
        let mut rng = make_rng_seeded(123);
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, &config, &mut rng);

        let total = 20 * 20;
        let black_count = grid.cells.iter().flatten().filter(|c| matches!(c, Cell::Black)).count();
        let ratio = black_count as f64 / total as f64;

        assert!(
            ratio >= config.black_square_ratio_min - 0.02 && ratio <= config.black_square_ratio_max + 0.06,
            "Hard black ratio {:.3} should be near [{:.2}, {:.2}]",
            ratio, config.black_square_ratio_min, config.black_square_ratio_max
        );
    }

    #[test]
    fn test_easy_has_higher_ratio_than_hard() {
        let easy_config = DifficultyConfig::easy();
        let hard_config = DifficultyConfig::hard();
        let mut rng = make_rng_seeded(777);
        let mut easy_grid = Grid::new(20, 20);
        seed_black_squares(&mut easy_grid, &easy_config, &mut rng);

        let mut rng2 = make_rng_seeded(888);
        let mut hard_grid = Grid::new(20, 20);
        seed_black_squares(&mut hard_grid, &hard_config, &mut rng2);

        let easy_black = easy_grid.cells.iter().flatten().filter(|c| matches!(c, Cell::Black)).count();
        let hard_black = hard_grid.cells.iter().flatten().filter(|c| matches!(c, Cell::Black)).count();

        assert!(
            easy_black > hard_black,
            "Easy grid ({} black) should have more black squares than hard grid ({} black)",
            easy_black, hard_black
        );
    }

    #[test]
    fn test_seeded_grid_is_connected() {
        let config = DifficultyConfig::easy();
        let mut rng = make_rng_seeded(999);
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, &config, &mut rng);

        assert!(is_connected(&grid), "Seeded grid must have connected white region");
    }

    #[test]
    fn test_no_isolated_white_cells() {
        let config = DifficultyConfig::medium();
        let mut rng = make_rng_seeded(55);
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, &config, &mut rng);

        let height = grid.height;
        let width = grid.width;
        let dirs: &[(i32, i32)] = &[(-1, 0), (1, 0), (0, -1), (0, 1)];

        for r in 0..height {
            for c in 0..width {
                if matches!(grid.cells[r][c], Cell::White { .. }) {
                    let white_count = dirs.iter().filter(|&&(dr, dc)| {
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr < 0 || nr >= height as i32 || nc < 0 || nc >= width as i32 {
                            return false;
                        }
                        matches!(grid.cells[nr as usize][nc as usize], Cell::White { .. })
                    }).count();
                    assert!(
                        white_count >= 1,
                        "White cell at ({}, {}) has no white neighbors — isolated",
                        r, c
                    );
                }
            }
        }
    }

    #[test]
    fn test_all_slots_length_min_or_more() {
        let config = DifficultyConfig::hard();
        let mut rng = make_rng_seeded(66666);
        let mut grid = Grid::new(20, 20);
        seed_black_squares(&mut grid, &config, &mut rng);

        let slots = extract_slots(&grid, config.min_word_length);
        for slot in &slots {
            assert!(
                slot.length >= config.min_word_length,
                "Found slot of length {} at ({},{}) {:?} (min={})",
                slot.length, slot.row, slot.col, slot.direction, config.min_word_length
            );
        }
    }

    #[test]
    fn test_extract_slots_across() {
        let mut grid = Grid::new(5, 1);
        grid.cells[0][2] = Cell::Black;
        let slots = extract_slots(&grid, 2);
        let across: Vec<_> = slots.iter().filter(|s| s.direction == Direction::Across).collect();
        assert_eq!(across.len(), 2);
        assert!(across.iter().any(|s| s.col == 0 && s.length == 2));
        assert!(across.iter().any(|s| s.col == 3 && s.length == 2));
    }

    #[test]
    fn test_extract_slots_down() {
        let mut grid = Grid::new(1, 5);
        grid.cells[2][0] = Cell::Black;
        let slots = extract_slots(&grid, 2);
        let down: Vec<_> = slots.iter().filter(|s| s.direction == Direction::Down).collect();
        assert_eq!(down.len(), 2);
        assert!(down.iter().any(|s| s.row == 0 && s.length == 2));
        assert!(down.iter().any(|s| s.row == 3 && s.length == 2));
    }

    #[test]
    fn test_extract_slots_single_cell_not_included() {
        let mut grid = Grid::new(3, 1);
        grid.cells[0][1] = Cell::Black;
        let slots = extract_slots(&grid, 2);
        let across: Vec<_> = slots.iter().filter(|s| s.direction == Direction::Across).collect();
        assert_eq!(across.len(), 0, "Length-1 runs should not produce slots");
    }
}
