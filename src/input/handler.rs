use macroquad::prelude::*;

use crate::game::state::PuzzleState;
use crate::grid::types::Direction;
use crate::render::grid::GridLayout;

/// Context for a double-click clue rating dialog (INTR-09).
pub struct RatingContext {
    pub word_id: i64,
    pub clue_text: String,
}

/// Persistent input state tracked across frames.
///
/// Enables double-click detection (INTR-09) and rating dialog display.
pub struct InputState {
    /// Time (in seconds) of the last left mouse click on a grid cell.
    pub last_click_time: f64,
    /// Grid (row, col) of the last left mouse click on a grid cell.
    pub last_click_pos: Option<(usize, usize)>,
    /// Active rating dialog, if a word was double-clicked.
    pub rating_dialog: Option<RatingContext>,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            last_click_time: -1.0,
            last_click_pos: None,
            rating_dialog: None,
        }
    }
}

/// Double-click threshold in seconds.
const DOUBLE_CLICK_THRESHOLD: f64 = 0.3;

/// Process all keyboard and mouse input for the current frame.
///
/// Mutates `state` in place. Called at the start of each Playing frame before drawing.
/// `input_state` tracks double-click detection and the active rating dialog.
pub fn process_input(state: &mut PuzzleState, layout: &GridLayout, input_state: &mut InputState) {
    // Keyboard: letter input per D-10/INTR-03
    while let Some(ch) = get_char_pressed() {
        if ch.is_alphabetic() {
            let upper = ch.to_ascii_uppercase();
            // IJ handling: if 'J' typed and current cell has 'I' and answer is IJ,
            // handle_ij_input promotes to IJ token and advances — do not also call
            // set_letter_and_advance (that would overwrite the IJ token with Single('J')).
            if upper == 'J' {
                let consumed = state.handle_ij_input();
                if !consumed {
                    state.set_letter_and_advance(upper);
                }
            } else {
                state.set_letter_and_advance(upper);
            }
        }
    }

    // Keyboard: backspace per D-11/INTR-04
    if is_key_pressed(KeyCode::Backspace) {
        state.backspace();
    }

    // Keyboard: arrow keys per D-12
    if is_key_pressed(KeyCode::Right) {
        state.move_cursor(Direction::Across, 1);
    }
    if is_key_pressed(KeyCode::Left) {
        state.move_cursor(Direction::Across, -1);
    }
    if is_key_pressed(KeyCode::Down) {
        state.move_cursor(Direction::Down, 1);
    }
    if is_key_pressed(KeyCode::Up) {
        state.move_cursor(Direction::Down, -1);
    }

    // Keyboard: Tab/Shift+Tab per D-12
    if is_key_pressed(KeyCode::Tab) {
        if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
            state.cycle_clue(-1);
        } else {
            state.cycle_clue(1);
        }
    }

    // Mouse: cell click per D-10/INTR-01, with double-click detection for INTR-09
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if let Some((row, col)) = layout.hit_test(mx, my, state.grid.height, state.grid.width) {
            let now = get_time();
            let is_double_click = input_state.last_click_pos == Some((row, col))
                && (now - input_state.last_click_time) <= DOUBLE_CLICK_THRESHOLD;

            if is_double_click {
                // Double-click: find the clue for the word at this cell (INTR-09)
                if let Some(clue_entry) = find_clue_at_cell(state, row, col) {
                    input_state.rating_dialog = Some(RatingContext {
                        word_id: clue_entry.word_id,
                        clue_text: clue_entry.text.clone(),
                    });
                }
                // Reset so next click is not counted as double
                input_state.last_click_pos = None;
                input_state.last_click_time = -1.0;
            } else {
                // Single click: normal cell selection
                state.handle_cell_click(row, col);
                input_state.last_click_time = now;
                input_state.last_click_pos = Some((row, col));
            }
        }
    }
}

/// Find the clue entry for the word at (row, col) in the current direction,
/// falling back to the other direction if none found. Used for double-click rating.
fn find_clue_at_cell<'a>(
    state: &'a PuzzleState,
    row: usize,
    col: usize,
) -> Option<&'a crate::game::state::ClueEntry> {
    // Try current direction first
    let dir = state.selected_direction;
    let entry = find_clue_in_direction(state, row, col, dir);
    if entry.is_some() {
        return entry;
    }
    // Fall back to other direction
    let other_dir = match dir {
        crate::grid::types::Direction::Across => crate::grid::types::Direction::Down,
        crate::grid::types::Direction::Down => crate::grid::types::Direction::Across,
    };
    find_clue_in_direction(state, row, col, other_dir)
}

fn find_clue_in_direction<'a>(
    state: &'a PuzzleState,
    row: usize,
    col: usize,
    dir: crate::grid::types::Direction,
) -> Option<&'a crate::game::state::ClueEntry> {
    let clues = match dir {
        crate::grid::types::Direction::Across => &state.across_clues,
        crate::grid::types::Direction::Down => &state.down_clues,
    };
    clues.iter().find(|e| {
        let s = &e.slot;
        match dir {
            crate::grid::types::Direction::Across => {
                s.row == row && s.col <= col && col < s.col + s.length
            }
            crate::grid::types::Direction::Down => {
                s.col == col && s.row <= row && row < s.row + s.length
            }
        }
    })
}
