use crate::paths::{aish_home, display_path};
use crate::shell::ShellTarget;
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub aish_home: PathBuf,
    pub shell: ShellTarget,
    pub model_path: PathBuf,
    pub model_download_dir: PathBuf,
    pub model_repo: String,
    pub model_file: String,
    pub auto_execute_safe: bool,
    pub confirm_risky: bool,
    pub confirm_network_download: bool,
    pub block_extreme_risk: bool,
    pub context_enabled: bool,
    pub context_commands: usize,
    pub logging_enabled: bool,
    pub runtime_backend: String,
    pub runtime_command: String,
    pub threads: usize,
    pub context_size: usize,
    pub temperature: f32,
    pub max_tokens: usize,
    pub generation_timeout_seconds: u64,
    pub natural_language_output: bool,
}

impl Config {
    pub fn load_or_create() -> Result<Self, String> {
        let home = aish_home()?;
        fs::create_dir_all(home.join("models")).map_err(|err| err.to_string())?;
        fs::create_dir_all(home.join("logs")).map_err(|err| err.to_string())?;
        fs::create_dir_all(home.join("cache")).map_err(|err| err.to_string())?;

        let path = home.join("config.toml");
        if !path.exists() {
            fs::write(&path, default_config_text(&home)).map_err(|err| err.to_string())?;
        }

        let text = fs::read_to_string(&path).map_err(|err| err.to_string())?;
        Ok(parse_config(&text, home))
    }

    pub fn history_path(&self) -> PathBuf {
        self.aish_home.join("history.tsv")
    }
}

fn parse_config(text: &str, home: PathBuf) -> Config {
    let mut config = Config {
        model_download_dir: home.join("models"),
        model_path: home.join("models").join("aish.gguf"),
        model_repo: "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF".to_string(),
        model_file: "qwen2.5-coder-1.5b-instruct-q2_k.gguf".to_string(),
        aish_home: home,
        shell: ShellTarget::default_for_platform(),
        auto_execute_safe: true,
        confirm_risky: true,
        confirm_network_download: true,
        block_extreme_risk: true,
        context_enabled: true,
        context_commands: 10,
        logging_enabled: false,
        runtime_backend: "mock".to_string(),
        runtime_command: "llama-cli".to_string(),
        threads: 4,
        context_size: 4096,
        temperature: 0.0,
        max_tokens: 256,
        generation_timeout_seconds: 60,
        natural_language_output: false,
    };

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }

        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = clean_value(raw_value);

        match key {
            "shell" => {
                if let Some(shell) = ShellTarget::parse(&value) {
                    config.shell = shell;
                }
            }
            "model_path" => config.model_path = crate::paths::expand_user_path(&value),
            "model_download_dir" => {
                config.model_download_dir = crate::paths::expand_user_path(&value)
            }
            "model_repo" => config.model_repo = value,
            "model_file" => config.model_file = value,
            "auto_execute_safe" => config.auto_execute_safe = parse_bool(&value, true),
            "confirm_risky" => config.confirm_risky = parse_bool(&value, true),
            "confirm_network_download" => {
                config.confirm_network_download = parse_bool(&value, true)
            }
            "block_extreme_risk" => config.block_extreme_risk = parse_bool(&value, true),
            "enabled" => {
                if raw_line.contains("[logging]") {
                    config.logging_enabled = parse_bool(&value, false);
                }
            }
            "context_enabled" => config.context_enabled = parse_bool(&value, true),
            "context_commands" => config.context_commands = parse_usize(&value, 10),
            "logging_enabled" => config.logging_enabled = parse_bool(&value, false),
            "runtime_backend" | "backend" => config.runtime_backend = value,
            "runtime_command" | "command" => config.runtime_command = value,
            "threads" => config.threads = parse_usize(&value, 4),
            "context_size" => config.context_size = parse_usize(&value, 4096),
            "temperature" => config.temperature = value.parse().unwrap_or(0.0),
            "max_tokens" => config.max_tokens = parse_usize(&value, 256),
            "generation_timeout_seconds" => {
                config.generation_timeout_seconds = value.parse().unwrap_or(60)
            }
            "natural_language_output" => config.natural_language_output = parse_bool(&value, false),
            _ => {}
        }
    }

    config
}

fn clean_value(raw: &str) -> String {
    let value = raw.split('#').next().unwrap_or("").trim();
    value.trim_matches('"').trim_matches('\'').to_string()
}

fn parse_bool(value: &str, fallback: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "1" | "on" => true,
        "false" | "no" | "0" | "off" => false,
        _ => fallback,
    }
}

fn parse_usize(value: &str, fallback: usize) -> usize {
    value.parse().unwrap_or(fallback)
}

fn default_config_text(home: &PathBuf) -> String {
    let model_download_dir = display_path(&home.join("models"));
    let model_path = display_path(&home.join("models").join("aish.gguf"));
    let shell = ShellTarget::default_for_platform().as_config_value();

    format!(
        "[aish]\n\
shell = \"{shell}\"\n\
model_download_dir = \"{model_download_dir}\"\n\
model_path = \"{model_path}\"\n\
model_repo = \"Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF\"\n\
model_file = \"qwen2.5-coder-1.5b-instruct-q2_k.gguf\"\n\
auto_execute_safe = true\n\
confirm_risky = true\n\
block_extreme_risk = true\n\
\n\
[context]\n\
context_enabled = true\n\
context_commands = 10\n\
suggest_after_seconds = 5\n\
\n\
[safety]\n\
confirm_sudo = true\n\
confirm_delete = true\n\
confirm_overwrite = true\n\
confirm_network_download = true\n\
block_root_delete = true\n\
block_disk_format = true\n\
\n\
[logging]\n\
logging_enabled = false\n\
log_commands = false\n\
log_model_prompts = false\n\
\n\
[runtime]\n\
# Use \"mock\" for development without a GGUF model.\n\
# Packaged AiSH bundles llama-cli and downloads the GGUF model on first use.\n\
runtime_backend = \"llama.cpp\"\n\
runtime_command = \"llama-cli\"\n\
threads = 4\n\
context_size = 4096\n\
temperature = 0.0\n\
max_tokens = 256\n\
generation_timeout_seconds = 60\n\
\n\
[output]\n\
# false prints command output unchanged; true summarizes it in natural language.\n\
natural_language_output = false\n"
    )
}
