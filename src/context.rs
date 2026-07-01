use std::env;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SystemContext {
    pub user: String,
    pub host: String,
    pub os: String,
    pub family: String,
    pub arch: String,
    pub cwd: PathBuf,
    pub home: Option<PathBuf>,
}

impl SystemContext {
    pub fn capture() -> Self {
        Self {
            user: current_user(),
            host: current_host(),
            os: env::consts::OS.to_string(),
            family: env::consts::FAMILY.to_string(),
            arch: env::consts::ARCH.to_string(),
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            home: current_home(),
        }
    }

    pub fn prompt_prefix(&self, shell: &str) -> String {
        format!(
            "{}@{} {} {} {}",
            self.user,
            self.host,
            self.short_cwd(),
            shell,
            self.os_label()
        )
    }

    pub fn os_label(&self) -> String {
        format!("{}/{}", self.os, self.arch)
    }

    pub fn short_cwd(&self) -> String {
        shorten_path(&self.cwd, self.home.as_deref())
    }

    pub fn describe_lines(&self) -> Vec<String> {
        let mut lines = vec![
            format!("user: {}", self.user),
            format!("host: {}", self.host),
            format!("os: {}", self.os),
            format!("os family: {}", self.family),
            format!("arch: {}", self.arch),
            format!("cwd: {}", self.cwd.display()),
        ];

        if let Some(home) = &self.home {
            lines.push(format!("home: {}", home.display()));
        }

        lines
    }
}

fn current_user() -> String {
    first_env(&["USER", "USERNAME", "LOGNAME"])
        .or_else(|| {
            current_home().and_then(|home| {
                home.file_name()
                    .map(|value| value.to_string_lossy().into_owned())
                    .filter(|value| !value.is_empty())
            })
        })
        .unwrap_or_else(|| "unknown-user".to_string())
}

fn current_host() -> String {
    first_env(&["HOSTNAME", "COMPUTERNAME"]).unwrap_or_else(|| "local".to_string())
}

fn current_home() -> Option<PathBuf> {
    first_env(&["HOME", "USERPROFILE"]).map(PathBuf::from)
}

fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

fn shorten_path(path: &Path, home: Option<&Path>) -> String {
    if let Some(home) = home {
        if path == home {
            return "~".to_string();
        }

        if let Ok(rest) = path.strip_prefix(home) {
            let rest = rest.to_string_lossy();
            if rest.is_empty() {
                return "~".to_string();
            }
            return format!("~/{}", rest);
        }
    }

    path.to_string_lossy().into_owned()
}
