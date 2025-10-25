use crate::utils::console::detect_editor;
use crate::utils::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub gist: Option<GistConfig>,
    pub gitlab: Option<GitLabConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub prompt_file: PathBuf,
    #[serde(default)]
    pub prompt_dirs: Vec<PathBuf>,
    pub editor: String,
    pub select_cmd: String,
    pub default_tags: Vec<String>,
    pub auto_sync: bool,
    pub sort_by: SortBy,
    pub color: bool,
    #[serde(default)]
    pub content_preview: bool,
    #[serde(default)]
    pub search_case_sensitive: bool,
    #[serde(default)]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GistConfig {
    pub file_name: String,
    #[serde(
        default,
        serialize_with = "crate::utils::format::serialize_option_string",
        deserialize_with = "crate::utils::format::deserialize_option_string"
    )]
    pub access_token: Option<String>,
    #[serde(
        default,
        serialize_with = "crate::utils::format::serialize_option_string",
        deserialize_with = "crate::utils::format::deserialize_option_string"
    )]
    pub gist_id: Option<String>,
    pub public: bool,
    pub auto_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    pub file_name: String,
    pub access_token: Option<String>,
    pub url: String,
    pub id: Option<i32>,
    pub visibility: String,
    pub auto_sync: bool,
    pub skip_ssl: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Recency,
    Title,
    Description,
    Updated,
}

impl Default for Config {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("promptheus");

        Self {
            general: GeneralConfig {
                prompt_file: config_dir.join("prompts.toml"),
                prompt_dirs: Vec::new(),
                editor: detect_editor(None),
                select_cmd: detect_best_select_command(),
                default_tags: Vec::new(),
                auto_sync: false,
                sort_by: SortBy::Recency,
                color: true,
                content_preview: true,
                search_case_sensitive: false,
                format: None,
            },
            gist: Some(GistConfig {
                file_name: String::new(),
                access_token: None,
                gist_id: None,
                public: false,
                auto_sync: false,
            }),
            gitlab: None,
        }
    }
}

/// Detect the best available selection command
fn detect_best_select_command() -> String {
    if cfg!(windows) {
        // On Windows, try to find a suitable selector
        if std::path::Path::new("C:\\Program Files\\Git\\usr\\bin\\fzf.exe").exists() {
            return "fzf".to_string();
        }
        "powershell".to_string() // Fallback to PowerShell
    } else {
        // On Unix-like systems, check for available tools
        if std::path::Path::new("/usr/bin/fzf").exists()
            || std::path::Path::new("/usr/local/bin/fzf").exists()
        {
            "fzf".to_string()
        } else if std::path::Path::new("/usr/bin/sk").exists()
            || std::path::Path::new("/usr/local/bin/sk").exists()
        {
            "sk".to_string()
        } else if std::path::Path::new("/usr/bin/peco").exists()
            || std::path::Path::new("/usr/local/bin/peco").exists()
        {
            "peco".to_string()
        } else {
            "fzf".to_string() // Default assumption
        }
    }
}

impl Config {
    pub fn load() -> AppResult<Self> {
        Self::load_custom(&Self::config_file_path())
    }

    pub fn ensure_config_exists() -> AppResult<()> {
        let config_path = Self::config_file_path();
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
        }
        Ok(())
    }

    pub fn load_custom(config_path: &std::path::Path) -> AppResult<Self> {
        if !config_path.exists() {
            let default_config = Config::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content =
            std::fs::read_to_string(config_path).map_err(|e| AppError::Io(e.to_string()))?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| AppError::System(format!("Failed to parse config file: {}", e)))?;

        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.general.editor.is_empty() {
            return Err(AppError::System("Editor cannot be empty".to_string()));
        }

        if self.general.select_cmd.is_empty() {
            return Err(AppError::System(
                "Select command cannot be empty".to_string(),
            ));
        }

        if let Some(gitlab) = &self.gitlab {
            if gitlab.url.is_empty() {
                return Err(AppError::System("GitLab URL cannot be empty".to_string()));
            }
            if gitlab.file_name.is_empty() {
                return Err(AppError::System(
                    "GitLab file name cannot be empty".to_string(),
                ));
            }
        }

        if let Some(gist) = &self.gist {
            // Only validate gist configuration if it's actually being used (has gist_id or non-empty file_name)
            if gist.gist_id.is_some() || !gist.file_name.is_empty() {
                if gist.file_name.is_empty() {
                    return Err(AppError::System(
                        "Gist file name cannot be empty when gist sync is configured".to_string(),
                    ));
                }

                // Validate access token availability if gist_id is set (for updating existing gist)
                if gist.gist_id.is_some() && gist.access_token.is_none() {
                    // Check environment variables
                    if std::env::var("PROMPTHEUS_GITHUB_ACCESS_TOKEN").is_err() {
                        return Err(AppError::System(
                            "GitHub access token is required for gist sync. Set it in config or use PROMPTHEUS_GITHUB_ACCESS_TOKEN environment variable".to_string()
                        ));
                    }
                }

                // Validate file name has proper extension
                if !std::path::Path::new(&gist.file_name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
                {
                    return Err(AppError::System(
                        "Gist file name should have .toml extension for proper prompt storage"
                            .to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn save(&self) -> AppResult<()> {
        let config_path = Self::config_file_path();

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e.to_string()))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::System(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&config_path, content).map_err(|e| AppError::Io(e.to_string()))?;

        Ok(())
    }

    pub fn config_file_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("promptheus")
            .join("config.toml")
    }
}
