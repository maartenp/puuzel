use std::path::PathBuf;

use crate::game::state::PuzzleState;

/// Returns the path for the save file.
/// Uses platform-appropriate directories (XDG on Linux, Application Support on macOS).
fn save_path() -> Option<PathBuf> {
    let proj = directories::ProjectDirs::from("", "", "puuzel")?;
    let dir = proj.data_dir();
    Some(dir.join("savegame.json"))
}

/// Save the current puzzle state to disk.
/// Creates the parent directory if it doesn't exist.
pub fn save_game(puzzle: &PuzzleState) {
    let path = match save_path() {
        Some(p) => p,
        None => {
            log::warn!("Could not determine save directory");
            return;
        }
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            log::warn!("Failed to create save directory: {}", e);
            return;
        }
    }

    match serde_json::to_string(puzzle) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                log::warn!("Failed to write save file: {}", e);
            } else {
                log::debug!("Game saved to {}", path.display());
            }
        }
        Err(e) => log::warn!("Failed to serialize game state: {}", e),
    }
}

/// Load a saved puzzle state from disk.
/// Returns None if no save file exists or if it can't be read/parsed.
pub fn load_game() -> Option<PuzzleState> {
    let path = save_path()?;
    let json = std::fs::read_to_string(&path).ok()?;
    match serde_json::from_str(&json) {
        Ok(puzzle) => {
            log::info!("Loaded saved game from {}", path.display());
            Some(puzzle)
        }
        Err(e) => {
            log::warn!("Failed to parse save file: {}", e);
            None
        }
    }
}

/// Delete the save file (e.g., when the puzzle is completed or a new one is started).
pub fn delete_save() {
    if let Some(path) = save_path() {
        if path.exists() {
            if let Err(e) = std::fs::remove_file(&path) {
                log::warn!("Failed to delete save file: {}", e);
            } else {
                log::debug!("Save file deleted");
            }
        }
    }
}
