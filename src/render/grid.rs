use macroquad::prelude::*;

use crate::game::state::PuzzleState;
use crate::grid::types::{Cell, Direction, LetterToken};

/// Layout geometry for the crossword grid panel.
pub struct GridLayout {
    /// X coordinate of the top-left corner of the grid
    pub origin_x: f32,
    /// Y coordinate of the top-left corner of the grid
    pub origin_y: f32,
    /// Size (width and height) of each cell in pixels
    pub cell_size: f32,
}

impl GridLayout {
    /// Compute the grid layout for the current screen dimensions.
    ///
    /// The grid panel occupies the left 60% of the screen (D-01).
    /// Cells are square and sized to fit the available area, with a minimum of 32px (D-02).
    pub fn compute(grid_cols: usize, grid_rows: usize) -> Self {
        let panel_width = screen_width() * 0.60;
        let panel_height = screen_height();
        let padding = 16.0_f32;

        let available_w = panel_width - padding * 2.0;
        let available_h = panel_height - padding * 2.0;

        let cell_w = if grid_cols > 0 { available_w / grid_cols as f32 } else { 32.0 };
        let cell_h = if grid_rows > 0 { available_h / grid_rows as f32 } else { 32.0 };
        let cell_size = cell_w.min(cell_h).max(32.0);

        // Center the grid within the panel area
        let grid_total_w = cell_size * grid_cols as f32;
        let grid_total_h = cell_size * grid_rows as f32;
        let origin_x = padding + (available_w - grid_total_w) / 2.0;
        let origin_y = padding + (available_h - grid_total_h) / 2.0;

        GridLayout {
            origin_x,
            origin_y,
            cell_size,
        }
    }

    /// Convert a mouse position to a grid cell (row, col).
    ///
    /// Returns `None` if the position is outside the grid bounds.
    pub fn hit_test(&self, mx: f32, my: f32, rows: usize, cols: usize) -> Option<(usize, usize)> {
        let rel_x = mx - self.origin_x;
        let rel_y = my - self.origin_y;

        if rel_x < 0.0 || rel_y < 0.0 {
            return None;
        }

        let col = (rel_x / self.cell_size) as usize;
        let row = (rel_y / self.cell_size) as usize;

        if col >= cols || row >= rows {
            return None;
        }

        Some((row, col))
    }
}

/// Returns all (row, col) pairs belonging to the active word (the word under the cursor
/// in the current direction).
fn cells_in_active_word(state: &PuzzleState) -> Vec<(usize, usize)> {
    let (sel_row, sel_col) = match state.selected_cell {
        Some(c) => c,
        None => return vec![],
    };

    // Find the clue whose slot contains the selected cell in the selected direction
    let clues = match state.selected_direction {
        Direction::Across => &state.across_clues,
        Direction::Down => &state.down_clues,
    };

    for entry in clues {
        let slot = &entry.slot;
        let cells: Vec<(usize, usize)> = match slot.direction {
            Direction::Across => (slot.col..slot.col + slot.length)
                .map(|c| (slot.row, c))
                .collect(),
            Direction::Down => (slot.row..slot.row + slot.length)
                .map(|r| (r, slot.col))
                .collect(),
        };

        if cells.contains(&(sel_row, sel_col)) {
            return cells;
        }
    }

    vec![]
}

/// Draw the complete crossword grid.
///
/// Renders black cells, white cells with borders, the selected cell highlight,
/// active word highlight, clue numbers, and the user's entered letters.
pub fn draw_grid(state: &PuzzleState, layout: &GridLayout) {
    let cs = layout.cell_size;

    // Draw grid panel background (slightly lighter than pure black)
    draw_rectangle(
        0.0,
        0.0,
        screen_width() * 0.60,
        screen_height(),
        Color::from_rgba(20, 20, 20, 255),
    );

    let active_cells = cells_in_active_word(state);

    for row in 0..state.grid.height {
        for col in 0..state.grid.width {
            let x = layout.origin_x + col as f32 * cs;
            let y = layout.origin_y + row as f32 * cs;

            match &state.grid.cells[row][col] {
                Cell::Black => {
                    // Black cell: solid dark gray (#333)
                    draw_rectangle(x, y, cs, cs, Color::from_rgba(51, 51, 51, 255));
                }
                Cell::White { letter: answer } => {
                    let is_selected = state.selected_cell == Some((row, col));
                    let is_active_word = active_cells.contains(&(row, col));

                    // Fill color
                    let fill_color = if is_active_word && !is_selected {
                        Color::from_rgba(173, 216, 230, 255) // Light blue for active word (D-03)
                    } else if is_selected {
                        Color::from_rgba(210, 230, 255, 255) // Slightly lighter blue for selected cell
                    } else {
                        WHITE
                    };

                    draw_rectangle(x, y, cs, cs, fill_color);

                    // Thin border for all white cells
                    draw_rectangle_lines(x, y, cs, cs, 1.0, DARKGRAY);

                    // Bold blue border for selected cell (D-03)
                    if is_selected {
                        draw_rectangle_lines(x, y, cs, cs, 3.0, Color::from_rgba(0, 100, 255, 255));
                    }

                    // Clue number in top-left corner (D-05)
                    if let Some(&num) = state.clue_numbers.get(&(row, col)) {
                        let num_str = num.to_string();
                        let font_size = (cs * 0.25).max(10.0).min(14.0);
                        draw_text_ex(
                            &num_str,
                            x + 2.0,
                            y + font_size + 1.0,
                            TextParams {
                                font_size: font_size as u16,
                                color: Color::from_rgba(60, 60, 60, 255),
                                ..Default::default()
                            },
                        );
                    }

                    // Draw user letter (or answer letter in selected cell for visibility)
                    let user_letter = &state.user_grid[row][col];
                    if let Some(token) = user_letter {
                        draw_letter(x, y, cs, token, BLACK);
                    }

                    // If the answer has IJ and user hasn't typed yet, we don't pre-fill
                    // (answers only shown on the answer grid, not the user grid, per D-13)
                    let _ = answer; // suppress unused variable warning
                }
            }
        }
    }
}

/// Draw a single letter token centered in a cell.
fn draw_letter(cell_x: f32, cell_y: f32, cell_size: f32, token: &LetterToken, color: Color) {
    let center_x = cell_x + cell_size / 2.0;
    let center_y = cell_y + cell_size / 2.0;

    match token {
        LetterToken::Single(ch) => {
            let text = ch.to_string();
            let font_size = (cell_size * 0.6) as u16;
            let dims = measure_text(&text, None, font_size, 1.0);
            draw_text_ex(
                &text,
                center_x - dims.width / 2.0,
                center_y + dims.height / 2.0,
                TextParams {
                    font_size,
                    color,
                    font_scale: 1.0,
                    font_scale_aspect: 1.0,
                    ..Default::default()
                },
            );
        }
        LetterToken::IJ => {
            // IJ digraph: compress horizontally to fit in single cell (D-04)
            let text = "IJ";
            let font_size = (cell_size * 0.6) as u16;
            let font_scale_aspect = 0.65_f32;
            // Measure at normal scale then account for compression
            let dims = measure_text(text, None, font_size, 1.0);
            draw_text_ex(
                text,
                center_x - (dims.width * font_scale_aspect) / 2.0,
                center_y + dims.height / 2.0,
                TextParams {
                    font_size,
                    color,
                    font_scale: 1.0,
                    font_scale_aspect,
                    ..Default::default()
                },
            );
        }
    }
}
