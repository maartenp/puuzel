use std::sync::mpsc;
use std::thread;

/// Spawn a background thread that fetches the remote version.txt from GitHub Pages.
///
/// Returns a Receiver that will produce:
/// - `Some(version_string)` if the fetch succeeded
/// - `None` if any error occurred (offline, timeout, DNS failure, etc.)
///
/// On any error, the check silently skips — never blocks startup or shows an error.
pub fn spawn_version_check() -> mpsc::Receiver<Option<String>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result: Option<String> = (|| {
            // ureq 3.x: read_to_string() takes no arguments and returns Result<String>
            let body = ureq::get("https://raw.githubusercontent.com/maartenp/puuzel/main/version.txt")
                .call()
                .ok()?
                .body_mut()
                .read_to_string()
                .ok()?;
            Some(body.trim().to_string())
        })();
        tx.send(result).ok();
    });
    rx
}
