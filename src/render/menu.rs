use macroquad::prelude::*;
use crate::grid::types::Difficulty;

/// Button dimensions and layout constants
const BUTTON_WIDTH: f32 = 300.0;
const BUTTON_HEIGHT: f32 = 70.0;
const BUTTON_GAP: f32 = 20.0;
const BUTTON_FONT_SIZE: f32 = 28.0;
const TITLE_FONT_SIZE: f32 = 64.0;

/// Draw the difficulty selection menu screen.
///
/// Shows the "Puuzel" title and three large Dutch-labeled buttons:
/// "Makkelijk" (Easy), "Middel" (Medium), "Moeilijk" (Hard).
///
/// Returns `Some(Difficulty)` if a button was clicked this frame, or `None`.
pub fn draw_menu_screen() -> Option<Difficulty> {
    clear_background(BLACK);

    let screen_w = screen_width();
    let screen_h = screen_height();

    // Draw title centered at top third
    let title = "Puuzel";
    let title_dims = measure_text(title, None, TITLE_FONT_SIZE as u16, 1.0);
    let title_x = (screen_w - title_dims.width) / 2.0;
    let title_y = screen_h / 3.0;
    draw_text(title, title_x, title_y, TITLE_FONT_SIZE, WHITE);

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
        let text_dims = measure_text(label, None, BUTTON_FONT_SIZE as u16, 1.0);
        let text_x = btn_x + (BUTTON_WIDTH - text_dims.width) / 2.0;
        let text_y = btn_y + (BUTTON_HEIGHT + text_dims.height) / 2.0;
        draw_text(label, text_x, text_y, BUTTON_FONT_SIZE, BLACK);

        if hovered && mouse_clicked {
            result = Some(*difficulty);
        }
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
    let font_size: f32 = 32.0;
    let dims = measure_text(text, None, font_size as u16, 1.0);
    let text_x = (screen_w - dims.width) / 2.0;
    let text_y = screen_h / 2.0 + dims.height / 2.0;

    draw_text(text, text_x, text_y, font_size, WHITE);
}
