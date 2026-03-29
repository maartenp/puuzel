mod grid;
mod db;
mod game;
mod render;
mod input;
mod update;
mod paths;

use macroquad::prelude::*;
use game::state::GameState;
use game::history::WordHistory;
use grid::types::DifficultyConfig;
use paths::resolve_data_path;

fn window_conf() -> Conf {
    Conf {
        window_title: "Puuzel".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    env_logger::init();

    render::init_font().await;

    let db_path = resolve_data_path("puuzel.db");
    log::info!("Using database: {}", db_path.display());

    let version_rx = update::spawn_version_check();
    let mut update_available: Option<String> = None;
    let mut update_dismissed = false;

    let mut state = GameState::DifficultySelection;
    let mut word_history = WordHistory::new();
    let mut input_state = input::handler::InputState::new();
    let mut clue_panel_state = render::clue_panel::CluePanelState::new();
    let mut test_mode = false;

    loop {
        clear_background(BLACK);

        // Check for version result (non-blocking) — D-04, D-06
        if update_available.is_none() && !update_dismissed {
            if let Ok(remote) = version_rx.try_recv() {
                if let Some(ref v) = remote {
                    if v != env!("CARGO_PKG_VERSION") {
                        update_available = Some(v.clone());
                    }
                }
                // If None (network failed) or same version, do nothing
            }
        }

        state = match state {
            GameState::DifficultySelection => {
                if let Some(diff) = render::menu::draw_menu_screen(&mut test_mode) {
                    let path = db_path.clone();
                    let exclude: std::collections::HashSet<i64> = word_history.recent_ids().collect();
                    let is_test_mode = test_mode;
                    let (tx, rx) = std::sync::mpsc::channel();
                    std::thread::spawn(move || {
                        let conn = match db::open_database(&path) {
                            Ok(c) => c,
                            Err(e) => { tx.send(Err(e.to_string())).ok(); return; }
                        };
                        let config = match diff {
                            grid::types::Difficulty::Easy => DifficultyConfig::easy(),
                            grid::types::Difficulty::Medium => DifficultyConfig::medium(),
                            grid::types::Difficulty::Hard => DifficultyConfig::hard(),
                        };
                        let gen_result = if is_test_mode {
                            grid::generator::generate_grid_test_mode(&conn, &config, &exclude)
                        } else {
                            grid::generator::generate_grid(&conn, &config, &exclude)
                        }.map_err(|e| e.to_string());
                        match gen_result {
                            Ok(filled) => {
                                let puzzle = game::state::PuzzleState::from_filled_grid(filled, &conn, is_test_mode);
                                tx.send(puzzle).ok();
                            }
                            Err(e) => { tx.send(Err(e)).ok(); }
                        }
                    });
                    GameState::Generating { rx }
                } else {
                    GameState::DifficultySelection
                }
            }
            GameState::Generating { rx } => {
                render::menu::draw_generating_screen();
                match rx.try_recv() {
                    Ok(Ok(puzzle)) => {
                        // Add used words to history
                        let word_ids: Vec<i64> = puzzle.across_clues.iter()
                            .chain(puzzle.down_clues.iter())
                            .map(|c| c.word_id)
                            .collect();
                        word_history.add_all(word_ids.into_iter());
                        clue_panel_state = render::clue_panel::CluePanelState::new();
                        GameState::Playing(puzzle)
                    }
                    Ok(Err(e)) => {
                        log::error!("Generation failed: {}", e);
                        GameState::DifficultySelection
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        GameState::Generating { rx }
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        log::error!("Generation thread panicked");
                        GameState::DifficultySelection
                    }
                }
            }
            GameState::Playing(mut puzzle) => {
                let layout = render::grid::GridLayout::compute(puzzle.grid.width, puzzle.grid.height);
                input::handler::process_input(&mut puzzle, &layout, &mut input_state);
                render::grid::draw_grid(&puzzle, &layout);
                let mut new_puzzle_requested = false;
                // Buttons above the grid
                if let Some(action) = render::grid::draw_buttons() {
                    match action {
                        render::clue_panel::PanelAction::NewPuzzle => {
                            new_puzzle_requested = true;
                        }
                        render::clue_panel::PanelAction::Check => {
                            puzzle.check_errors();
                        }
                        _ => {}
                    }
                }
                // Determine which clues the hovered grid cell belongs to
                let (mx, my) = macroquad::input::mouse_position();
                let hover_clues = layout.hit_test(mx, my, puzzle.grid.height, puzzle.grid.width)
                    .map(|(r, c)| puzzle.clue_numbers_at(r, c))
                    .unwrap_or((None, None));
                // Clue panel (scrollable, with double-click rating)
                if let Some(action) = render::clue_panel::draw_clue_panel(&puzzle, &mut clue_panel_state, hover_clues, &input_state.clue_ratings) {
                    match action {
                        render::clue_panel::PanelAction::ClueClick(click) => {
                            puzzle.select_clue(&click.slot);
                        }
                        render::clue_panel::PanelAction::RateClue { word_id, clue_text } => {
                            let current_rating = input_state.clue_ratings.get(&word_id).copied();
                            input_state.rating_dialog = Some(input::handler::RatingContext {
                                word_id,
                                clue_text,
                                opened_at: macroquad::time::get_time(),
                                current_rating,
                            });
                        }
                        _ => {}
                    }
                }
                // Double-click rating dialog (INTR-09)
                if let Some(ref ctx) = input_state.rating_dialog {
                    if let Some(rating_result) = render::overlay::draw_rating_dialog(&ctx.clue_text, ctx.opened_at, ctx.current_rating) {
                        if let Some(thumbs_up) = rating_result {
                            log::info!("Clue rated: word_id={}, thumbs_up={}", ctx.word_id, thumbs_up);
                            input_state.clue_ratings.insert(ctx.word_id, thumbs_up);
                        }
                        input_state.rating_dialog = None;
                    }
                }
                if new_puzzle_requested {
                    GameState::DifficultySelection
                } else if puzzle.is_complete() {
                    GameState::Congratulations(puzzle)
                } else {
                    GameState::Playing(puzzle)
                }
            }
            GameState::Congratulations(puzzle) => {
                let layout = render::grid::GridLayout::compute(puzzle.grid.width, puzzle.grid.height);
                render::grid::draw_grid(&puzzle, &layout);
                let _ = render::clue_panel::draw_clue_panel(&puzzle, &mut clue_panel_state, (None, None), &input_state.clue_ratings);
                if render::overlay::draw_congratulations() {
                    // "Nieuwe puzzel" clicked — return to difficulty selection (FLOW-02)
                    GameState::DifficultySelection
                } else {
                    GameState::Congratulations(puzzle)
                }
            }
        };
        // Show update notification overlay on top of everything (D-05)
        if let Some(ref _version) = update_available {
            if render::overlay::draw_update_notification() {
                update_dismissed = true;
                update_available = None;
            }
        }

        next_frame().await;
    }
}
