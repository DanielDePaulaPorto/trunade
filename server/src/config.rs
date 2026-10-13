use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

pub const CONFIG_FILE_NAME: &str = "trunade.toml";

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Language {
    #[serde(default)]
    pub aliases: Vec<String>,
    /// File name the cell source is written to, e.g. `main.cpp`.
    pub source: String,
    /// Optional compile step. Placeholders: `{source}`, `{binary}`, `{flags}`.
    #[serde(default)]
    pub compile: Option<Vec<String>>,
    /// Run step. Same placeholders as `compile`.
    pub run: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub timeout: Duration,
    pub compile_timeout: Duration,
    pub output_limit_bytes: usize,
    /// Command that opens the Markdown file at a line. Placeholders: `{file}`, `{line}`.
    pub editor: Vec<String>,
    pub languages: BTreeMap<String, Language>,
}

/// Every field optional, so a file only needs to state what it overrides.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    timeout_seconds: Option<f64>,
    compile_timeout_seconds: Option<f64>,
    output_limit_bytes: Option<usize>,
    editor: Option<Vec<String>>,
    #[serde(default)]
    languages: BTreeMap<String, Language>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            compile_timeout: Duration::from_secs(60),
            output_limit_bytes: 1024 * 1024,
            editor: strings(&["zed", "{file}:{line}"]),
            languages: default_languages(),
        }
    }
}

impl Config {
    /// Defaults, then the user config, then `trunade.toml` beside the document,
    /// or only `explicit` when given.
    pub fn load(document: &Path, explicit: Option<&Path>) -> Result<Self> {
        let mut config = Self::default();
        for path in config_candidates(document, explicit) {
            if path.exists() || explicit == Some(path.as_path()) {
                config.merge(read_config_file(&path)?);
            }
        }
        Ok(config)
    }

    pub fn language(&self, fence_language: &str) -> Option<(&str, &Language)> {
        let wanted = fence_language.to_lowercase();
        self.languages
            .iter()
            .find(|(name, language)| {
                **name == wanted || language.aliases.iter().any(|alias| *alias == wanted)
            })
            .map(|(name, language)| (name.as_str(), language))
    }

    fn merge(&mut self, file: ConfigFile) {
        if let Some(seconds) = file.timeout_seconds {
            self.timeout = Duration::from_secs_f64(seconds);
        }
        if let Some(seconds) = file.compile_timeout_seconds {
            self.compile_timeout = Duration::from_secs_f64(seconds);
        }
        if let Some(bytes) = file.output_limit_bytes {
            self.output_limit_bytes = bytes;
        }
        if let Some(editor) = file.editor {
            self.editor = editor;
        }
        self.languages.extend(file.languages);
    }
}

/// `~/.config/trunade`: user-wide config and the page's saved preferences.
pub fn user_config_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config/trunade"))
}

fn config_candidates(document: &Path, explicit: Option<&Path>) -> Vec<PathBuf> {
    if let Some(path) = explicit {
        return vec![path.to_path_buf()];
    }
    let user_config = user_config_dir().map(|dir| dir.join("config.toml"));
    let beside_document = document.parent().map(|dir| dir.join(CONFIG_FILE_NAME));
    user_config.into_iter().chain(beside_document).collect()
}

fn read_config_file(path: &Path) -> Result<ConfigFile> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing config {}", path.display()))
}

/// Replaces placeholders; `{flags}` expands to zero or more arguments.
pub fn expand(template: &[String], placeholders: &[(&str, &str)], flags: &[String]) -> Vec<String> {
    let mut expanded = Vec::with_capacity(template.len() + flags.len());
    for argument in template {
        if argument == "{flags}" {
            expanded.extend(flags.iter().cloned());
            continue;
        }
        let mut value = argument.clone();
        for (name, replacement) in placeholders {
            value = value.replace(&format!("{{{name}}}"), replacement);
        }
        expanded.push(value);
    }
    expanded
}

fn default_languages() -> BTreeMap<String, Language> {
    BTreeMap::from([
        (
            "cpp".to_string(),
            Language {
                aliases: strings(&["c++", "cxx", "cc"]),
                source: "main.cpp".to_string(),
                compile: Some(strings(&[
                    "g++-16",
                    "-std=c++23",
                    "-Wall",
                    "-Wextra",
                    "-fdiagnostics-color=always",
                    "{flags}",
                    "{source}",
                    "-o",
                    "{binary}",
                ])),
                run: strings(&["{binary}"]),
            },
        ),
        (
            "c".to_string(),
            Language {
                aliases: vec![],
                source: "main.c".to_string(),
                compile: Some(strings(&[
                    "gcc-16",
                    "-std=c17",
                    "-Wall",
                    "-Wextra",
                    "-fdiagnostics-color=always",
                    "{flags}",
                    "{source}",
                    "-o",
                    "{binary}",
                ])),
                run: strings(&["{binary}"]),
            },
        ),
        (
            "python".to_string(),
            Language {
                aliases: strings(&["py", "python3"]),
                source: "main.py".to_string(),
                compile: None,
                run: strings(&["python3", "-u", "{flags}", "{source}"]),
            },
        ),
    ])
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_is_found_by_name_or_alias_ignoring_case() {
        let config = Config::default();
        assert_eq!(config.language("cpp").unwrap().0, "cpp");
        assert_eq!(config.language("C++").unwrap().0, "cpp");
        assert_eq!(config.language("py").unwrap().0, "python");
        assert!(config.language("text").is_none());
    }

    #[test]
    fn flags_placeholder_expands_to_many_arguments() {
        let template = strings(&["cc", "{flags}", "{source}", "-o", "{binary}"]);
        let expanded = expand(
            &template,
            &[("source", "main.c"), ("binary", "main")],
            &strings(&["-O2", "-g"]),
        );
        assert_eq!(expanded, strings(&["cc", "-O2", "-g", "main.c", "-o", "main"]));
    }

    #[test]
    fn config_file_overrides_only_what_it_states() {
        let mut config = Config::default();
        config.merge(
            toml::from_str(
                r#"
                timeout_seconds = 2.5
                [languages.cpp]
                source = "main.cpp"
                compile = ["clang++", "{source}", "-o", "{binary}"]
                run = ["{binary}"]
                "#,
            )
            .unwrap(),
        );
        assert_eq!(config.timeout, Duration::from_millis(2500));
        assert_eq!(config.compile_timeout, Duration::from_secs(60));
        assert_eq!(config.languages["cpp"].compile.as_ref().unwrap()[0], "clang++");
        assert!(config.languages.contains_key("python"));
    }
}
