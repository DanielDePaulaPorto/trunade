//! Display preferences chosen in the page (fonts, theme, text width…), kept in
//! `~/.config/trunade/preferences.toml` so every lesson opens the same way,
//! whatever the port. The page validates the values; the server only stores them.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const PREFERENCES_FILE_NAME: &str = "preferences.toml";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Preferences {
    pub theme: Option<String>,
    pub code_theme: Option<String>,
    pub locale: Option<String>,
    pub font_scale: Option<f64>,
    pub text_width: Option<String>,
    pub title_font: Option<String>,
    pub body_font: Option<String>,
    pub code_font: Option<String>,
}

/// A missing file is an empty set of preferences; a broken one too, with a warning.
pub fn load(path: &Path) -> Preferences {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|error| {
            eprintln!("warning: ignoring {}: {error}", path.display());
            Preferences::default()
        }),
        Err(_) => Preferences::default(),
    }
}

/// Writes atomically, so a crash never leaves a half-written file.
pub fn save(path: &Path, preferences: &Preferences) -> Result<()> {
    let dir = path.parent().context("preferences path has no directory")?;
    std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let text = format!(
        "# Trunade! display preferences, written by the settings panel.\n{}",
        toml::to_string(preferences)?
    );
    let mut file = tempfile::NamedTempFile::new_in(dir)?;
    file.write_all(text.as_bytes())?;
    file.persist(path).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn default_path() -> Option<PathBuf> {
    crate::config::user_config_dir().map(|dir| dir.join(PREFERENCES_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_gives_empty_preferences() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(&dir.path().join("none.toml")), Preferences::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/preferences.toml");
        let preferences = Preferences {
            title_font: Some("gloock".into()),
            font_scale: Some(1.3),
            ..Preferences::default()
        };
        save(&path, &preferences).unwrap();
        assert_eq!(load(&path), preferences);
    }

    #[test]
    fn broken_file_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.toml");
        std::fs::write(&path, "title_font = [").unwrap();
        assert_eq!(load(&path), Preferences::default());
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let preferences: Preferences = toml::from_str("body_font = \"inter\"\nfuture = 1").unwrap();
        assert_eq!(preferences.body_font.as_deref(), Some("inter"));
    }
}
