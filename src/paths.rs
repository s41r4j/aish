use std::env;
use std::path::{Path, PathBuf};

pub fn aish_home() -> Result<PathBuf, String> {
    if let Ok(value) = env::var("AISH_HOME") {
        return Ok(PathBuf::from(value));
    }

    #[cfg(windows)]
    {
        if let Ok(appdata) = env::var("APPDATA") {
            return Ok(PathBuf::from(appdata).join("AiSH"));
        }

        if let Ok(profile) = env::var("USERPROFILE") {
            return Ok(PathBuf::from(profile).join(".aish"));
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".aish"));
        }
    }

    Err("could not resolve AiSH home directory".to_string())
}

pub fn expand_user_path(value: &str) -> PathBuf {
    if value == "~" {
        if let Some(home) = user_home() {
            return home;
        }
    }

    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = user_home() {
            return home.join(rest);
        }
    }

    PathBuf::from(value)
}

pub fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn user_home() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        env::var("USERPROFILE").ok().map(PathBuf::from)
    }

    #[cfg(not(windows))]
    {
        env::var("HOME").ok().map(PathBuf::from)
    }
}
