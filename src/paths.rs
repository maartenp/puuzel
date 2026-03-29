use std::path::PathBuf;

/// Resolve a data file path, checking locations in order:
/// 1. Flatpak install path (`/app/share/puuzel/`)
/// 2. Next to the executable (Windows / portable installs)
/// 3. `data/` relative to working directory (development)
pub fn resolve_data_path(filename: &str) -> PathBuf {
    // Flatpak
    let flatpak = PathBuf::from(format!("/app/share/puuzel/{}", filename));
    if flatpak.exists() {
        return flatpak;
    }

    // Next to the executable (Windows .exe distribution)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let beside_exe = exe_dir.join("data").join(filename);
            if beside_exe.exists() {
                return beside_exe;
            }
        }
    }

    // Development fallback
    PathBuf::from(format!("data/{}", filename))
}
