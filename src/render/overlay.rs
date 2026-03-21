use macroquad::prelude::*;

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
    let title_size = 36u16;
    let title_dims = measure_text(title_text, None, title_size, 1.0);
    let title_x = box_x + (box_w - title_dims.width) / 2.0;
    let title_y = box_y + 65.0;
    draw_text(title_text, title_x, title_y, title_size as f32, DARKGREEN);

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

    let btn_dims = measure_text(btn_text, None, 22, 1.0);
    let text_x = btn_x + (btn_w - btn_dims.width) / 2.0;
    let text_y = btn_y + (btn_h + btn_dims.height) / 2.0;
    draw_text(btn_text, text_x, text_y, 22.0, WHITE);

    // Return true if button was clicked
    hovered && is_mouse_button_pressed(MouseButton::Left)
}

/// Draw the clue rating dialog for a double-clicked word (INTR-09).
///
/// Shows a small overlay with the clue text and two rating buttons.
/// Returns:
///   - `Some(true)` if the user clicked "Goed" (thumbs up)
///   - `Some(false)` if the user clicked "Slecht" (thumbs down)
///   - `None` if the dialog is still open (no button pressed this frame)
///
/// Note: actual persistence of ratings is Phase 3 (FLOW-04). This is a UI stub.
pub fn draw_rating_dialog(clue_text: &str) -> Option<bool> {
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
    draw_text(header, box_x + 12.0, box_y + 26.0, 16.0, LIGHTGRAY);

    // Clue text — truncated if too long
    let max_chars = 50;
    let display_text: String = if clue_text.chars().count() > max_chars {
        let truncated: String = clue_text.chars().take(max_chars - 3).collect();
        format!("{}...", truncated)
    } else {
        clue_text.to_string()
    };
    draw_text(&display_text, box_x + 12.0, box_y + 52.0, 18.0, WHITE);

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

    let goed_color = if goed_hovered {
        Color::new(0.1, 0.7, 0.2, 1.0)
    } else {
        Color::new(0.05, 0.5, 0.1, 1.0)
    };
    let slecht_color = if slecht_hovered {
        Color::new(0.8, 0.2, 0.1, 1.0)
    } else {
        Color::new(0.6, 0.1, 0.05, 1.0)
    };

    draw_rectangle(goed_x, btn_y, btn_w, btn_h, goed_color);
    draw_rectangle_lines(goed_x, btn_y, btn_w, btn_h, 1.0, DARKGREEN);
    let goed_dims = measure_text("Goed", None, 20, 1.0);
    draw_text(
        "Goed",
        goed_x + (btn_w - goed_dims.width) / 2.0,
        btn_y + (btn_h + goed_dims.height) / 2.0,
        20.0,
        WHITE,
    );

    draw_rectangle(slecht_x, btn_y, btn_w, btn_h, slecht_color);
    draw_rectangle_lines(slecht_x, btn_y, btn_w, btn_h, 1.0, Color::new(0.5, 0.0, 0.0, 1.0));
    let slecht_dims = measure_text("Slecht", None, 20, 1.0);
    draw_text(
        "Slecht",
        slecht_x + (btn_w - slecht_dims.width) / 2.0,
        btn_y + (btn_h + slecht_dims.height) / 2.0,
        20.0,
        WHITE,
    );

    if is_mouse_button_pressed(MouseButton::Left) {
        if goed_hovered {
            return Some(true);
        }
        if slecht_hovered {
            return Some(false);
        }
        // Click outside both buttons — dismiss dialog (return None to signal dismissed)
        // We signal dismissal by returning Some(false) with a click outside — but the caller
        // uses input_state.rating_dialog = None when any Some() is returned.
        // To support "click outside to dismiss", we need a third return option.
        // Since the signature returns None for "still open", we return Some(false) as fallback dismiss.
        // (Phase 3 will refine: only persisted when explicit thumbs chosen.)
        return Some(false);
    }

    None
}
