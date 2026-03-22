use macroquad::prelude::*;

use crate::game::state::PuzzleState;
use crate::grid::types::{Direction, Slot};

/// Returned when a clue is clicked in the panel.
pub struct ClueClickAction {
    pub slot: Slot,
    pub word_id: i64,
}

/// Actions returned from the clue panel (buttons or clue clicks).
pub enum PanelAction {
    ClueClick(ClueClickAction),
    NewPuzzle,
    Check,
}

const PANEL_START_RATIO: f32 = 0.57;
const PANEL_WIDTH_RATIO: f32 = 0.42;
const HEADER_FONT_SIZE: u16 = 18;
const CLUE_FONT_SIZE: u16 = 15;
const NUM_FONT_SIZE: u16 = 17;
const LINE_HEIGHT: f32 = 20.0;
const PADDING: f32 = 8.0;
const SECTION_GAP: f32 = 8.0;
const SCROLL_SPEED: f32 = 40.0;

/// Persistent scroll state for the two clue sections.
pub struct CluePanelState {
    pub across_scroll: f32,
    pub down_scroll: f32,
}

impl CluePanelState {
    pub fn new() -> Self {
        CluePanelState {
            across_scroll: 0.0,
            down_scroll: 0.0,
        }
    }
}

/// Draw the clue panel with "Horizontaal" and "Verticaal" sections.
pub fn draw_clue_panel(state: &PuzzleState, panel_state: &mut CluePanelState) -> Option<PanelAction> {
    let panel_x = screen_width() * PANEL_START_RATIO;
    let panel_w = screen_width() * PANEL_WIDTH_RATIO;
    let panel_h = screen_height();

    // Panel background
    draw_rectangle(panel_x, 0.0, panel_w, panel_h, Color::from_rgba(30, 30, 30, 255));

    // Separator line
    draw_line(panel_x, 0.0, panel_x, panel_h, 2.0, Color::from_rgba(80, 80, 80, 255));

    let (mx, my) = mouse_position();
    let clicked = is_mouse_button_pressed(MouseButton::Left);
    let (_scroll_x, scroll_y) = mouse_wheel();

    let mut result: Option<PanelAction> = None;
    let mut y = PADDING;

    // === Horizontaal section ===
    let active_clue_num = state.active_clue_number();
    y = draw_section_header("Horizontaal", panel_x, y, panel_w);

    let across_start = y;
    let half_remaining = (panel_h - across_start - SECTION_GAP) / 2.0;

    // Handle scroll for Horizontaal if mouse is in this section
    let mouse_in_across = mx >= panel_x && mx <= panel_x + panel_w
        && my >= across_start && my < across_start + half_remaining;
    if mouse_in_across && scroll_y.abs() > 0.0 {
        panel_state.across_scroll -= scroll_y * SCROLL_SPEED;
    }

    if let Some(action) = draw_flowing_clues(
        &state.across_clues, Direction::Across, active_clue_num, state.selected_direction,
        panel_x + PADDING, across_start, panel_w - PADDING * 2.0, half_remaining,
        &mut panel_state.across_scroll, mx, my, clicked,
    ) {
        if result.is_none() { result = Some(PanelAction::ClueClick(action)); }
    }

    // === Verticaal section ===
    y = across_start + half_remaining + SECTION_GAP;
    y = draw_section_header("Verticaal", panel_x, y, panel_w);

    let down_start = y;
    let remaining = panel_h - down_start - PADDING;

    // Handle scroll for Verticaal if mouse is in this section
    let mouse_in_down = mx >= panel_x && mx <= panel_x + panel_w
        && my >= down_start && my <= panel_h;
    if mouse_in_down && scroll_y.abs() > 0.0 {
        panel_state.down_scroll -= scroll_y * SCROLL_SPEED;
    }

    if let Some(action) = draw_flowing_clues(
        &state.down_clues, Direction::Down, active_clue_num, state.selected_direction,
        panel_x + PADDING, down_start, panel_w - PADDING * 2.0, remaining,
        &mut panel_state.down_scroll, mx, my, clicked,
    ) {
        if result.is_none() { result = Some(PanelAction::ClueClick(action)); }
    }

    result
}

fn draw_section_header(title: &str, panel_x: f32, y: f32, panel_w: f32) -> f32 {
    draw_rectangle(panel_x, y, panel_w, LINE_HEIGHT + 2.0, Color::from_rgba(50, 50, 80, 255));
    draw_text_ex(title, panel_x + PADDING, y + HEADER_FONT_SIZE as f32,
        TextParams { font_size: HEADER_FONT_SIZE, color: WHITE, ..Default::default() });
    y + LINE_HEIGHT + 4.0
}

/// A positioned piece of text belonging to a clue.
struct Span {
    clue_idx: usize,
    draw_x: f32,
    draw_y: f32,
    text: String,
    span_width: f32,
    is_number: bool,
}

/// Compute the full flowing layout of clues (ignoring viewport clipping).
/// Returns spans and total content height.
fn layout_flowing_clues(
    clues: &[crate::game::state::ClueEntry],
    x: f32,
    start_y: f32,
    width: f32,
) -> (Vec<Span>, f32) {
    let space_w = measure_text(" ", None, CLUE_FONT_SIZE, 1.0).width;
    let mut spans: Vec<Span> = Vec::new();
    let mut cx: f32 = 0.0;
    let mut cy: f32 = start_y;

    for (i, entry) in clues.iter().enumerate() {
        let num_str = entry.number.to_string();
        let clue_words: Vec<&str> = entry.text.split_whitespace().collect();

        let num_w = measure_text(&num_str, None, NUM_FONT_SIZE, 1.0).width;
        let first_word = clue_words.first().copied().unwrap_or("");
        let first_w = if !first_word.is_empty() {
            measure_text(first_word, None, CLUE_FONT_SIZE, 1.0).width
        } else {
            0.0
        };
        let combined_w = num_w + space_w + first_w;

        // Separator space from previous clue
        if cx > 0.0 {
            cx += space_w * 2.0;
        }

        // Wrap if number+first_word doesn't fit
        if cx + combined_w > width && cx > 0.0 {
            cy += LINE_HEIGHT;
            cx = 0.0;
        }

        // Number
        spans.push(Span {
            clue_idx: i, draw_x: x + cx, draw_y: cy,
            text: num_str, span_width: num_w, is_number: true,
        });
        cx += num_w + space_w;

        // First word (same line as number)
        if !first_word.is_empty() {
            spans.push(Span {
                clue_idx: i, draw_x: x + cx, draw_y: cy,
                text: first_word.to_string(), span_width: first_w, is_number: false,
            });
            cx += first_w;
        }

        // Remaining words
        for word in clue_words.iter().skip(1) {
            let ww = measure_text(word, None, CLUE_FONT_SIZE, 1.0).width;
            if cx + space_w + ww > width && cx > 0.0 {
                cy += LINE_HEIGHT;
                cx = 0.0;
            } else {
                cx += space_w;
            }
            spans.push(Span {
                clue_idx: i, draw_x: x + cx, draw_y: cy,
                text: word.to_string(), span_width: ww, is_number: false,
            });
            cx += ww;
        }
    }

    let total_height = cy - start_y + LINE_HEIGHT;
    (spans, total_height)
}

/// Draw clues in flowing paragraph with scroll support.
#[allow(clippy::too_many_arguments)]
fn draw_flowing_clues(
    clues: &[crate::game::state::ClueEntry],
    direction: Direction,
    active_clue_num: Option<u32>,
    selected_direction: Direction,
    x: f32,
    start_y: f32,
    width: f32,
    view_height: f32,
    scroll_offset: &mut f32,
    mx: f32,
    my: f32,
    clicked: bool,
) -> Option<ClueClickAction> {
    let mut result: Option<ClueClickAction> = None;

    // Compute full layout
    let (spans, total_height) = layout_flowing_clues(clues, x, start_y, width);

    // Clamp scroll offset
    let max_scroll = (total_height - view_height).max(0.0);
    *scroll_offset = scroll_offset.clamp(0.0, max_scroll);

    let scroll = *scroll_offset;

    // Draw scroll indicator if content overflows
    if max_scroll > 0.0 {
        let panel_x = screen_width() * PANEL_START_RATIO;
        let panel_w = screen_width() * PANEL_WIDTH_RATIO;
        let bar_x = panel_x + panel_w - 4.0;
        let bar_h = (view_height / total_height) * view_height;
        let bar_y = start_y + (scroll / max_scroll) * (view_height - bar_h);
        draw_rectangle(bar_x, bar_y, 3.0, bar_h, Color::from_rgba(100, 100, 100, 180));
    }

    // Second pass: highlights + click (with scroll offset applied)
    for (i, entry) in clues.iter().enumerate() {
        let is_active = Some(entry.number) == active_clue_num
            && selected_direction == direction;

        let clue_spans: Vec<&Span> = spans.iter().filter(|s| s.clue_idx == i).collect();
        if clue_spans.is_empty() { continue; }

        // Group spans by line, applying scroll offset
        let mut lines: Vec<(f32, f32, f32, f32)> = Vec::new();
        let mut line_y = clue_spans[0].draw_y - scroll;
        let mut lx_min = clue_spans[0].draw_x;
        let mut lx_max = clue_spans[0].draw_x + clue_spans[0].span_width;

        for span in &clue_spans {
            let sy = span.draw_y - scroll;
            if (sy - line_y).abs() < 1.0 {
                lx_min = lx_min.min(span.draw_x);
                lx_max = lx_max.max(span.draw_x + span.span_width);
            } else {
                lines.push((lx_min, line_y, lx_max, line_y + LINE_HEIGHT));
                line_y = sy;
                lx_min = span.draw_x;
                lx_max = span.draw_x + span.span_width;
            }
        }
        lines.push((lx_min, line_y, lx_max, line_y + LINE_HEIGHT));

        // Only process lines that are visible
        let visible_lines: Vec<&(f32, f32, f32, f32)> = lines.iter()
            .filter(|&&(_, ly, _, ry)| ry > start_y && ly < start_y + view_height)
            .collect();

        if visible_lines.is_empty() { continue; }

        let is_hovered = visible_lines.iter().any(|&&(lx, ly, rx, ry)| {
            mx >= lx - 2.0 && mx <= rx + 2.0 && my >= ly && my <= ry
        });

        if is_active {
            for &&(lx, ly, rx, ry) in &visible_lines {
                let clip_y = ly.max(start_y);
                let clip_ry = ry.min(start_y + view_height);
                if clip_ry > clip_y {
                    draw_rectangle(lx - 2.0, clip_y, rx - lx + 4.0, clip_ry - clip_y, Color::from_rgba(70, 130, 180, 255));
                }
            }
        } else if is_hovered {
            for &&(lx, ly, rx, ry) in &visible_lines {
                let clip_y = ly.max(start_y);
                let clip_ry = ry.min(start_y + view_height);
                if clip_ry > clip_y {
                    draw_rectangle(lx - 2.0, clip_y, rx - lx + 4.0, clip_ry - clip_y, Color::from_rgba(50, 50, 70, 255));
                }
            }
        }

        if is_hovered && clicked {
            result = Some(ClueClickAction {
                slot: entry.slot.clone(),
                word_id: entry.word_id,
            });
        }
    }

    // Third pass: draw text (with scroll offset, clipped to view)
    for span in &spans {
        let sy = span.draw_y - scroll;
        let text_y = sy + CLUE_FONT_SIZE as f32;

        // Skip if outside visible area
        if sy + LINE_HEIGHT < start_y || sy > start_y + view_height {
            continue;
        }

        let entry = &clues[span.clue_idx];
        let is_active = Some(entry.number) == active_clue_num
            && selected_direction == direction;

        let (size, color) = if span.is_number {
            let c = if is_active { Color::from_rgba(255, 255, 100, 255) } else { WHITE };
            (NUM_FONT_SIZE, c)
        } else {
            let c = if is_active { WHITE } else { Color::from_rgba(200, 200, 200, 255) };
            (CLUE_FONT_SIZE, c)
        };

        draw_text_ex(
            &span.text, span.draw_x, text_y,
            TextParams { font_size: size, color, ..Default::default() },
        );
    }

    result
}
