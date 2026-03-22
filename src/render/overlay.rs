use macroquad::prelude::*;
use crate::render::{measure, text_params};

/// Draw the congratulations overlay when the puzzle is complete (D-16).
///
/// Renders a semi-transparent dark overlay with "Gefeliciteerd!" text and a
/// "Nieuwe puzzel" button. Returns true if the "Nieuwe puzzel" button was clicked,
/// indicating the user wants to start a new puzzle (FLOW-01, FLOW-02).
pub fn draw_congratulations() -> bool {
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent dark overlay over the whole screen
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.75));

    // Centered white dialog box: 400px wide, 200px tall
    let box_w = 400.0f32;
    let box_h = 200.0f32;
    let box_x = (sw - box_w) / 2.0;
    let box_y = (sh - box_h) / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, WHITE);
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, DARKGRAY);

    // "Gefeliciteerd!" text — centered in box, dark green per D-16
    let title_text = "Gefeliciteerd!";
    let title_size: u16 = 36;
    let title_dims = measure(title_text, title_size);
    let title_x = box_x + (box_w - title_dims.width) / 2.0;
    let title_y = box_y + 65.0;
    draw_text_ex(title_text, title_x, title_y, text_params(title_size, DARKGREEN));

    // "Nieuwe puzzel" button — centered horizontally, below the title
    let btn_text = "Nieuwe puzzel";
    let btn_w = 200.0f32;
    let btn_h = 48.0f32;
    let btn_x = box_x + (box_w - btn_w) / 2.0;
    let btn_y = box_y + box_h - btn_h - 24.0;

    let (mx, my) = mouse_position();
    let hovered = mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;

    let btn_color = if hovered {
        Color::new(0.2, 0.5, 0.9, 1.0) // brighter blue on hover
    } else {
        Color::new(0.15, 0.35, 0.7, 1.0) // steel blue
    };

    draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_color);
    draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 1.5, DARKBLUE);

    let btn_dims = measure(btn_text, 22);
    let text_x = btn_x + (btn_w - btn_dims.width) / 2.0;
    let text_y = btn_y + (btn_h + btn_dims.height) / 2.0;
    draw_text_ex(btn_text, text_x, text_y, text_params(22, WHITE));

    // Return true if button was clicked
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Draw an update notification overlay (D-05). Returns true if dismissed.
///
/// Shows a Dutch message telling the user to run `flatpak update` to update.
/// The user can dismiss by clicking the "OK" button.
pub fn draw_update_notification() -> bool {
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent dark overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.6));

    // Dialog box
    let box_w = 500.0f32;
    let box_h = 200.0f32;
    let box_x = (sw - box_w) / 2.0;
    let box_y = (sh - box_h) / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, WHITE);
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, DARKGRAY);

    // Title
    let title = "Nieuwe versie beschikbaar";
    let title_size: u16 = 28;
    let title_dims = measure(title, title_size);
    let title_x = box_x + (box_w - title_dims.width) / 2.0;
    let title_y = box_y + 55.0;
    draw_text_ex(title, title_x, title_y, text_params(title_size, DARKBLUE));

    // Body text (D-05 Dutch message)
    let body = "Voer 'flatpak update' uit om bij te werken.";
    let body_size: u16 = 20;
    let body_dims = measure(body, body_size);
    let body_x = box_x + (box_w - body_dims.width) / 2.0;
    let body_y = box_y + 95.0;
    draw_text_ex(body, body_x, body_y, text_params(body_size, DARKGRAY));

    // "OK" button to dismiss
    let btn_text = "OK";
    let btn_w = 120.0f32;
    let btn_h = 44.0f32;
    let btn_x = box_x + (box_w - btn_w) / 2.0;
    let btn_y = box_y + box_h - btn_h - 24.0;

    let (mx, my) = mouse_position();
    let hovered = mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;

    let btn_color = if hovered {
        Color::new(0.2, 0.5, 0.9, 1.0)
    } else {
        Color::new(0.15, 0.35, 0.7, 1.0)
    };

    draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_color);
    // Button text centered
    let btn_text_dims = measure(btn_text, 22);
    let btn_text_x = btn_x + (btn_w - btn_text_dims.width) / 2.0;
    let btn_text_y = btn_y + btn_h / 2.0 + btn_text_dims.height / 2.0;
    draw_text_ex(btn_text, btn_text_x, btn_text_y, text_params(22, WHITE));

    // Click detection
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Draw the clue rating dialog for a double-clicked word (INTR-09).
///
/// Shows a small overlay with the clue text and two rating buttons.
/// Returns:
///   - `Some(Some(true))` if the user clicked "Goed" (thumbs up)
///   - `Some(Some(false))` if the user clicked "Slecht" (thumbs down)
///   - `Some(None)` if clicked outside (dismiss without rating change)
///   - `None` if the dialog is still open (no button pressed this frame)
pub fn draw_rating_dialog(clue_text: &str, opened_at: f64, current_rating: Option<bool>) -> Option<Option<bool>> {
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent overlay
    draw_rectangle(0.0, 0.0, sw, sh, Color::new(0.0, 0.0, 0.0, 0.55));

    // Small centered dialog box
    let box_w = 380.0f32;
    let box_h = 160.0f32;
    let box_x = (sw - box_w) / 2.0;
    let box_y = (sh - box_h) / 2.0;

    draw_rectangle(box_x, box_y, box_w, box_h, Color::new(0.15, 0.15, 0.2, 1.0));
    draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.0, GRAY);

    // Header label
    let header = "Aanwijzing beoordelen:";
    draw_text_ex(header, box_x + 12.0, box_y + 26.0, text_params(16, LIGHTGRAY));

    // Clue text — truncated if too long
    let max_chars = 50;
    let display_text: String = if clue_text.chars().count() > max_chars {
        let truncated: String = clue_text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        clue_text.to_string()
    };
    draw_text_ex(&display_text, box_x + 12.0, box_y + 52.0, text_params(18, WHITE));

    // Rating buttons: "Goed" and "Slecht"
    let btn_w = 120.0f32;
    let btn_h = 44.0f32;
    let btn_y = box_y + box_h - btn_h - 16.0;

    let goed_x = box_x + box_w / 2.0 - btn_w - 8.0;
    let slecht_x = box_x + box_w / 2.0 + 8.0;

    let (mx, my) = mouse_position();

    let goed_hovered = mx >= goed_x && mx <= goed_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
    let slecht_hovered =
        mx >= slecht_x && mx <= slecht_x + btn_w && my >= btn_y && my <= btn_y + btn_h;

    let is_rated_good = current_rating == Some(true);
    let is_rated_bad = current_rating == Some(false);

    let goed_color = if goed_hovered {
        Color::new(0.1, 0.7, 0.2, 1.0)
    } else if is_rated_good {
        Color::new(0.1, 0.65, 0.15, 1.0)
    } else {
        Color::new(0.05, 0.5, 0.1, 1.0)
    };
    let slecht_color = if slecht_hovered {
        Color::new(0.8, 0.2, 0.1, 1.0)
    } else if is_rated_bad {
        Color::new(0.75, 0.15, 0.08, 1.0)
    } else {
        Color::new(0.6, 0.1, 0.05, 1.0)
    };

    draw_rectangle(goed_x, btn_y, btn_w, btn_h, goed_color);
    let goed_border = if is_rated_good { 3.0 } else { 1.0 };
    let goed_border_color = if is_rated_good { WHITE } else { DARKGREEN };
    draw_rectangle_lines(goed_x, btn_y, btn_w, btn_h, goed_border, goed_border_color);
    let goed_dims = measure("Goed", 20);
    draw_text_ex(
        "Goed",
        goed_x + (btn_w - goed_dims.width) / 2.0,
        btn_y + (btn_h + goed_dims.height) / 2.0,
        text_params(20, WHITE),
    );

    draw_rectangle(slecht_x, btn_y, btn_w, btn_h, slecht_color);
    let slecht_border = if is_rated_bad { 3.0 } else { 1.0 };
    let slecht_border_color = if is_rated_bad { WHITE } else { Color::new(0.5, 0.0, 0.0, 1.0) };
    draw_rectangle_lines(slecht_x, btn_y, btn_w, btn_h, slecht_border, slecht_border_color);
    let slecht_dims = measure("Slecht", 20);
    draw_text_ex(
        "Slecht",
        slecht_x + (btn_w - slecht_dims.width) / 2.0,
        btn_y + (btn_h + slecht_dims.height) / 2.0,
        text_params(20, WHITE),
    );

    // Skip input during the opening frame to prevent the double-click
    // that opened the dialog from immediately dismissing it.
    let grace_elapsed = get_time() - opened_at > 0.05;

    if grace_elapsed && is_mouse_button_pressed(MouseButton::Left) {
        if goed_hovered {
            return Some(Some(true));
        }
        if slecht_hovered {
            return Some(Some(false));
        }
        // Click outside both buttons — dismiss without changing rating
        return Some(None);
    }

    None
}
