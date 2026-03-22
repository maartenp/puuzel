use macroquad::prelude::*;
use crate::grid::types::Difficulty;
use crate::render::{measure, text_params};

/// Button dimensions and layout constants
const BUTTON_WIDTH: f32 = 300.0;
const BUTTON_HEIGHT: f32 = 70.0;
const BUTTON_GAP: f32 = 20.0;
const BUTTON_FONT_SIZE: f32 = 28.0;
const TITLE_FONT_SIZE: f32 = 64.0;

/// Draw the difficulty selection menu screen.
///
/// Shows the "Puuzel" title, three large Dutch-labeled buttons, and a test mode toggle.
/// "Makkelijk" (Easy), "Middel" (Medium), "Moeilijk" (Hard).
///
/// Returns `Some(Difficulty)` if a button was clicked this frame, or `None`.
/// The `test_mode` parameter is toggled in-place when the user clicks the toggle.
pub fn draw_menu_screen(test_mode: &mut bool) -> Option<Difficulty> {
    clear_background(BLACK);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Draw title centered at top third
    let title = "Puuzel";
    let title_dims = measure(title, TITLE_FONT_SIZE as u16);
    let title_x = (screen_w - title_dims.width) / 2.0;
    let title_y = screen_h / 3.0;
    draw_text_ex(title, title_x, title_y, text_params(TITLE_FONT_SIZE as u16, WHITE));

    // Button labels and corresponding difficulties
    let buttons = [
        ("Makkelijk", Difficulty::Easy),
        ("Middel", Difficulty::Medium),
        ("Moeilijk", Difficulty::Hard),
    ];

    let total_buttons_height =
        buttons.len() as f32 * BUTTON_HEIGHT + (buttons.len() - 1) as f32 * BUTTON_GAP;
    let buttons_start_y = screen_h / 2.0;

    let mouse_pos = mouse_position();
    let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);

    let mut result = None;

    for (i, (label, difficulty)) in buttons.iter().enumerate() {
        let btn_x = (screen_w - BUTTON_WIDTH) / 2.0;
        let btn_y = buttons_start_y + i as f32 * (BUTTON_HEIGHT + BUTTON_GAP);

        // Detect hover for subtle visual feedback
        let hovered = mouse_pos.0 >= btn_x
            && mouse_pos.0 <= btn_x + BUTTON_WIDTH
            && mouse_pos.1 >= btn_y
            && mouse_pos.1 <= btn_y + BUTTON_HEIGHT;

        // Button fill: slightly gray on hover, white otherwise
        let btn_color = if hovered { Color::from_rgba(220, 220, 220, 255) } else { WHITE };

        draw_rectangle(btn_x, btn_y, BUTTON_WIDTH, BUTTON_HEIGHT, btn_color);

        // Center text in button
        let text_dims = measure(label, BUTTON_FONT_SIZE as u16);
        let text_x = btn_x + (BUTTON_WIDTH - text_dims.width) / 2.0;
        let text_y = btn_y + (BUTTON_HEIGHT + text_dims.height) / 2.0;
        draw_text_ex(label, text_x, text_y, text_params(BUTTON_FONT_SIZE as u16, BLACK));

        if hovered && mouse_clicked {
            result = Some(*difficulty);
        }
    }

    // Test mode toggle at bottom of menu
    let toggle_font_size: u16 = 20;
    let toggle_label = if *test_mode { "[x] Test modus (geen clues)" } else { "[ ] Test modus (geen clues)" };
    let toggle_dims = measure(toggle_label, toggle_font_size);
    let toggle_x = (screen_w - toggle_dims.width) / 2.0;
    let toggle_y = buttons_start_y + total_buttons_height + 40.0;

    let toggle_hovered = mouse_pos.0 >= toggle_x
        && mouse_pos.0 <= toggle_x + toggle_dims.width
        && mouse_pos.1 >= toggle_y - toggle_dims.height
        && mouse_pos.1 <= toggle_y;

    let toggle_color = if toggle_hovered { YELLOW } else { GRAY };
    draw_text_ex(toggle_label, toggle_x, toggle_y, text_params(toggle_font_size, toggle_color));

    if toggle_hovered && mouse_clicked {
        *test_mode = !*test_mode;
    }

    result
}

/// Draw the loading screen shown while a puzzle is being generated.
///
/// Shows "Puzzel wordt gemaakt..." centered on screen.
pub fn draw_generating_screen() {
    clear_background(BLACK);

    let screen_w = screen_width();
    let screen_h = screen_height();

    let text = "Puzzel wordt gemaakt...";
    let font_size: u16 = 32;
    let dims = measure(text, font_size);
    let text_x = (screen_w - dims.width) / 2.0;
    let text_y = screen_h / 2.0 + dims.height / 2.0;

    draw_text_ex(text, text_x, text_y, text_params(font_size, WHITE));
}
