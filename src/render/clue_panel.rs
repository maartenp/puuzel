use macroquad::prelude::*;

use crate::game::state::PuzzleState;
use crate::grid::types::{Direction, Slot};

/// Returned when a clue is clicked in the panel.
pub struct ClueClickAction {
    pub slot: Slot,
    pub word_id: i64,
}

/// Configuration constants for the clue panel.
const PANEL_START_RATIO: f32 = 0.62;
const PANEL_WIDTH_RATIO: f32 = 0.36;
const HEADER_FONT_SIZE: u16 = 22;
const CLUE_FONT_SIZE: u16 = 18;
const LINE_HEIGHT: f32 = 26.0;
const PADDING: f32 = 12.0;
const SECTION_GAP: f32 = 20.0;

/// Draw the clue panel with "Horizontaal" and "Verticaal" sections.
///
/// Returns `Some(ClueClickAction)` if a clue was clicked, `None` otherwise.
pub fn draw_clue_panel(state: &PuzzleState) -> Option<ClueClickAction> {
    let panel_x = screen_width() * PANEL_START_RATIO;
    let panel_w = screen_width() * PANEL_WIDTH_RATIO;
    let panel_h = screen_height();

    // Panel background
    draw_rectangle(panel_x, 0.0, panel_w, panel_h, Color::from_rgba(30, 30, 30, 255));

    // Separator line between grid and clue panel
    draw_line(
        panel_x,
        0.0,
        panel_x,
        panel_h,
        2.0,
        Color::from_rgba(80, 80, 80, 255),
    );

    let active_clue_num = state.active_clue_number();
    let (mx, my) = mouse_position();
    let clicked = is_mouse_button_pressed(MouseButton::Left);

    let mut result: Option<ClueClickAction> = None;
    let mut y = PADDING;

    // === Horizontaal section ===
    y = draw_section_header("Horizontaal", panel_x, y, panel_w);

    let across_scroll_area_start = y;
    let half_panel = (panel_h - across_scroll_area_start - SECTION_GAP) / 2.0;

    for entry in &state.across_clues {
        if y > across_scroll_area_start + half_panel {
            break; // Simple clipping — no scroll for now, just show as many as fit
        }

        let is_active = Some(entry.number) == active_clue_num
            && state.selected_direction == Direction::Across;

        let clue_result = draw_clue_item(
            entry.number,
            &entry.text,
            &entry.slot,
            entry.word_id,
            panel_x,
            y,
            panel_w,
            is_active,
            mx,
            my,
            clicked,
        );

        if clue_result.is_some() {
            result = clue_result;
        }

        y += LINE_HEIGHT;
    }

    // === Verticaal section ===
    y = across_scroll_area_start + half_panel + SECTION_GAP;
    y = draw_section_header("Verticaal", panel_x, y, panel_w);

    for entry in &state.down_clues {
        if y > panel_h - LINE_HEIGHT {
            break; // Simple clipping
        }

        let is_active = Some(entry.number) == active_clue_num
            && state.selected_direction == Direction::Down;

        let clue_result = draw_clue_item(
            entry.number,
            &entry.text,
            &entry.slot,
            entry.word_id,
            panel_x,
            y,
            panel_w,
            is_active,
            mx,
            my,
            clicked,
        );

        if clue_result.is_some() {
            result = clue_result;
        }

        y += LINE_HEIGHT;
    }

    result
}

/// Draw a section header ("Horizontaal" or "Verticaal") and return the new y position.
fn draw_section_header(title: &str, panel_x: f32, y: f32, panel_w: f32) -> f32 {
    // Header background
    draw_rectangle(
        panel_x,
        y,
        panel_w,
        LINE_HEIGHT + 4.0,
        Color::from_rgba(50, 50, 80, 255),
    );

    draw_text_ex(
        title,
        panel_x + PADDING,
        y + HEADER_FONT_SIZE as f32,
        TextParams {
            font_size: HEADER_FONT_SIZE,
            color: WHITE,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            ..Default::default()
        },
    );

    y + LINE_HEIGHT + 4.0 + 4.0 // header height + small gap
}

/// Draw a single clue item. Returns `Some(ClueClickAction)` if clicked.
#[allow(clippy::too_many_arguments)]
fn draw_clue_item(
    number: u32,
    text: &str,
    slot: &Slot,
    word_id: i64,
    panel_x: f32,
    y: f32,
    panel_w: f32,
    is_active: bool,
    mx: f32,
    my: f32,
    clicked: bool,
) -> Option<ClueClickAction> {
    // Active clue highlighted with blue background (D-07)
    if is_active {
        draw_rectangle(
            panel_x,
            y,
            panel_w,
            LINE_HEIGHT,
            Color::from_rgba(70, 130, 180, 255),
        );
    }

    // Hover highlight (lighter background)
    let is_hovered = mx >= panel_x
        && mx <= panel_x + panel_w
        && my >= y
        && my <= y + LINE_HEIGHT;

    if is_hovered && !is_active {
        draw_rectangle(
            panel_x,
            y,
            panel_w,
            LINE_HEIGHT,
            Color::from_rgba(60, 60, 80, 255),
        );
    }

    // Number (bold-ish — slightly larger)
    let num_str = format!("{:2}.", number);
    draw_text_ex(
        &num_str,
        panel_x + PADDING,
        y + CLUE_FONT_SIZE as f32,
        TextParams {
            font_size: CLUE_FONT_SIZE,
            color: if is_active {
                Color::from_rgba(255, 255, 100, 255) // Yellow number on active clue
            } else {
                Color::from_rgba(200, 200, 200, 255)
            },
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            ..Default::default()
        },
    );

    // Clue text
    let num_width = 38.0_f32;
    let text_x = panel_x + PADDING + num_width;
    let max_text_w = panel_w - PADDING - num_width - PADDING;

    // Truncate text if too long to fit
    let display_text = truncate_text(text, CLUE_FONT_SIZE, max_text_w);

    draw_text_ex(
        &display_text,
        text_x,
        y + CLUE_FONT_SIZE as f32,
        TextParams {
            font_size: CLUE_FONT_SIZE,
            color: if is_active { WHITE } else { Color::from_rgba(220, 220, 220, 255) },
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            ..Default::default()
        },
    );

    // Click detection (D-08)
    if clicked && is_hovered {
        return Some(ClueClickAction {
            slot: slot.clone(),
            word_id,
        });
    }

    None
}

/// Truncate text to fit within max_width pixels.
fn truncate_text(text: &str, font_size: u16, max_width: f32) -> String {
    let full_dims = measure_text(text, None, font_size, 1.0);
    if full_dims.width <= max_width {
        return text.to_string();
    }

    // Binary search or linear scan for truncation point
    let ellipsis = "...";
    let ellipsis_dims = measure_text(ellipsis, None, font_size, 1.0);
    let target_width = max_width - ellipsis_dims.width;

    let mut result = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}", result, ch);
        let dims = measure_text(&candidate, None, font_size, 1.0);
        if dims.width > target_width {
            break;
        }
        result.push(ch);
    }

    format!("{}{}", result, ellipsis)
}
