use std::collections::HashMap;
use macroquad::prelude::*;

use crate::game::state::PuzzleState;
use crate::grid::types::Direction;
use crate::render::grid::GridLayout;

/// Context for a clue rating dialog (INTR-09).
pub struct RatingContext {
    pub word_id: i64,
    pub clue_text: String,
    /// Time when the dialog was opened — skip input for a short grace period
    /// to avoid the click being consumed as a dismiss.
    pub opened_at: f64,
    /// Existing rating if the clue was already rated (allows changing).
    pub current_rating: Option<bool>,
}

/// Persistent input state tracked across frames.
pub struct InputState {
    /// Active rating dialog, if a clue was double-clicked.
    pub rating_dialog: Option<RatingContext>,
    /// Ratings given during this session: word_id → thumbs_up.
    pub clue_ratings: HashMap<i64, bool>,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            rating_dialog: None,
            clue_ratings: HashMap::new(),
        }
    }
}

/// Process all keyboard and mouse input for the current frame.
///
/// Mutates `state` in place. Called at the start of each Playing frame before drawing.
pub fn process_input(state: &mut PuzzleState, layout: &GridLayout, _input_state: &mut InputState) {
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

    // Keyboard: delete key — clear current cell without moving
    if is_key_pressed(KeyCode::Delete) {
        state.delete_current();
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

    // Mouse: cell click per D-10/INTR-01
    if is_mouse_button_pressed(MouseButton::Left) {
        let (mx, my) = mouse_position();
        if let Some((row, col)) = layout.hit_test(mx, my, state.grid.height, state.grid.width) {
            state.handle_cell_click(row, col);
        }
    }
}
