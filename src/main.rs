mod grid;
mod db;
mod game;
mod render;
mod input;

use macroquad::prelude::*;
use game::state::GameState;
use game::history::WordHistory;
use grid::types::DifficultyConfig;
use std::path::PathBuf;

fn window_conf() -> Conf {
    Conf {
        window_title: "Puuzel".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    env_logger::init();

    let db_path = PathBuf::from("data/puuzel.db");
    let mut state = GameState::DifficultySelection;
    let mut word_history = WordHistory::new();
    let mut input_state = input::handler::InputState::new();
    let mut clue_panel_state = render::clue_panel::CluePanelState::new();

    loop {
        clear_background(BLACK);
        state = match state {
            GameState::DifficultySelection => {
                if let Some(diff) = render::menu::draw_menu_screen() {
                    let path = db_path.clone();
                    let exclude: std::collections::HashSet<i64> = word_history.recent_ids().collect();
                    let _ = exclude; // passed to generator via word history mechanism; reserved for future use
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
                        let gen_result = grid::generator::generate_grid(&conn, &config)
                            .map_err(|e| e.to_string());
                        match gen_result {
                            Ok(filled) => {
                                let puzzle = game::state::PuzzleState::from_filled_grid(filled, &conn);
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
                // Clue panel (scrollable)
                if let Some(action) = render::clue_panel::draw_clue_panel(&puzzle, &mut clue_panel_state) {
                    match action {
                        render::clue_panel::PanelAction::ClueClick(click) => {
                            puzzle.select_clue(&click.slot);
                        }
                        _ => {}
                    }
                }
                // Double-click rating dialog (INTR-09)
                if let Some(ref ctx) = input_state.rating_dialog {
                    if let Some(_thumbs_up) = render::overlay::draw_rating_dialog(&ctx.clue_text) {
                        log::info!("Clue rated: word_id={}, thumbs_up={}", ctx.word_id, _thumbs_up);
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
                let _ = render::clue_panel::draw_clue_panel(&puzzle, &mut clue_panel_state);
                if render::overlay::draw_congratulations() {
                    // "Nieuwe puzzel" clicked — return to difficulty selection (FLOW-02)
                    GameState::DifficultySelection
                } else {
                    GameState::Congratulations(puzzle)
                }
            }
        };
        next_frame().await;
    }
}
