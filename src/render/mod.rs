pub mod menu;
pub mod grid;
pub mod clue_panel;
pub mod overlay;

use macroquad::prelude::*;
use std::sync::OnceLock;

/// Global app font, loaded once at startup. Falls back to macroquad default if not set.
static APP_FONT: OnceLock<Font> = OnceLock::new();

/// Initialize the app font from a TTF file. Call once at startup.
///
/// Checks the Flatpak install path first (`/app/share/puuzel/DejaVuSans.ttf`),
/// falls back to the dev path (`data/DejaVuSans.ttf`).
pub async fn init_font() {
    let font_path = if std::path::Path::new("/app/share/puuzel/DejaVuSans.ttf").exists() {
        "/app/share/puuzel/DejaVuSans.ttf"
    } else {
        "data/DejaVuSans.ttf"
    };
    match load_ttf_font(font_path).await {
        Ok(mut font) => {
            font.set_filter(FilterMode::Nearest);
            let _ = APP_FONT.set(font);
            log::info!("Loaded font from {}", font_path);
        }
        Err(e) => {
            log::warn!("Failed to load font from {}: {} — using default", font_path, e);
        }
    }
}

/// Get the app font, or None to use macroquad default.
pub fn app_font() -> Option<&'static Font> {
    APP_FONT.get()
}

/// Convenience: create TextParams with the app font.
pub fn text_params(font_size: u16, color: Color) -> TextParams<'static> {
    let mut params = TextParams {
        font_size,
        color,
        ..Default::default()
    };
    if let Some(font) = app_font() {
        params.font = Some(font);
    }
    params
}

/// Convenience: measure text with the app font.
pub fn measure(text: &str, font_size: u16) -> TextDimensions {
    measure_text(text, app_font(), font_size, 1.0)
}
