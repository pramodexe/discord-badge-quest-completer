//! The Library's "Your games" list, saved as `library.json` in the same folder
//! as the games (see `games_base_dir`), so it comes back the next time the app
//! opens. The interface decides what goes in it; this side only checks that it
//! is the expected shape, keeps it to a sane size and writes it safely.

use std::fs;
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "library.json";
/// Far more than any real list (a game entry is about 1 KB).
const MAX_BYTES: usize = 5 * 1024 * 1024;

fn library_path() -> PathBuf {
    crate::games_base_dir().join(FILE_NAME)
}

/// `{"version": 1, "games": [...]}` and not too big.
fn validate(data: &str) -> Result<(), String> {
    if data.len() > MAX_BYTES {
        return Err(format!("Library is too large to save ({} bytes)", data.len()));
    }
    let value: serde_json::Value =
        serde_json::from_str(data).map_err(|e| format!("Library is not valid JSON: {}", e))?;
    match value.get("games") {
        Some(games) if games.is_array() => Ok(()),
        _ => Err("Library has no \"games\" list".to_string()),
    }
}

/// Returns the saved library, or `None` if nothing was saved yet. A damaged
/// file is moved aside to `library.json.bad` (so nothing is lost) and reported.
#[tauri::command]
pub async fn load_library() -> Result<Option<String>, String> {
    load_from(&library_path())
}

fn load_from(path: &Path) -> Result<Option<String>, String> {
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("Could not read {}: {}", path.display(), e)),
    };
    match validate(&data) {
        Ok(()) => Ok(Some(data)),
        Err(problem) => {
            let backup = path.with_extension("json.bad");
            let _ = fs::rename(path, &backup);
            Err(format!(
                "Your saved games list was damaged ({}). A copy was kept at {}; starting with an empty list.",
                problem,
                backup.display()
            ))
        }
    }
}

#[tauri::command]
pub async fn save_library(data: String) -> Result<(), String> {
    save_to(&library_path(), &data)
}

fn save_to(path: &Path, data: &str) -> Result<(), String> {
    validate(data)?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("Could not create {}: {}", dir.display(), e))?;
    }
    // Write a temporary file and swap it in, so a crash or power cut never
    // leaves a half-written library behind.
    let partial = path.with_extension("json.partial");
    fs::write(&partial, data).map_err(|e| format!("Could not save the games list: {}", e))?;
    fs::rename(&partial, path).map_err(|e| format!("Could not save the games list: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("dqc-library-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn checks_the_shape() {
        assert!(validate(r#"{"version":1,"games":[]}"#).is_ok());
        assert!(validate(r#"{"version":1,"games":[{"id":"1"}]}"#).is_ok());
        assert!(validate("[]").is_err());
        assert!(validate(r#"{"games":"nope"}"#).is_err());
        assert!(validate("not json").is_err());
    }

    #[test]
    fn saves_and_loads_back() {
        let path = temp_dir("roundtrip").join("library.json");
        assert_eq!(load_from(&path).unwrap(), None);
        let data = r#"{"version":1,"games":[{"id":"42","name":"Test"}]}"#;
        save_to(&path, data).unwrap();
        assert_eq!(load_from(&path).unwrap().as_deref(), Some(data));
        assert!(!path.with_extension("json.partial").exists());
    }

    #[test]
    fn refuses_to_save_bad_data() {
        let path = temp_dir("refuse").join("library.json");
        assert!(save_to(&path, "garbage").is_err());
        assert!(!path.exists());
    }

    #[test]
    fn moves_a_damaged_file_aside() {
        let path = temp_dir("damaged").join("library.json");
        fs::write(&path, "{ broken").unwrap();
        assert!(load_from(&path).is_err());
        assert!(!path.exists());
        assert_eq!(fs::read_to_string(path.with_extension("json.bad")).unwrap(), "{ broken");
    }
}
