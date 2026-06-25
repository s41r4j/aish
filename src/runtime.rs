use crate::config::Config;
use crate::history::HistoryEntry;
use crate::model::ensure_model;
use crate::prompt::{build_command_prompt, clean_model_output};
use crate::shell::ShellTarget;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Runtime {
    config: Config,
}

impl Runtime {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn generate(
        &self,
        shell: ShellTarget,
        request: &str,
        context: &[HistoryEntry],
    ) -> Result<String, String> {
        let backend = env::var("AISH_RUNTIME").unwrap_or_else(|_| self.config.runtime_backend.clone());

        if backend.eq_ignore_ascii_case("mock") {
            return Ok(mock_generate(shell, request));
        }

        if backend.eq_ignore_ascii_case("llama.cpp") || backend.eq_ignore_ascii_case("llama") {
            return self.generate_with_llama(shell, request, context);
        }

        Err(format!(
            "unsupported runtime backend '{backend}'. Use 'mock' or 'llama.cpp'"
        ))
    }

    fn generate_with_llama(
        &self,
        shell: ShellTarget,
        request: &str,
        _context: &[HistoryEntry],
    ) -> Result<String, String> {
        ensure_model(&self.config)?;

        let prompt = build_command_prompt(shell, request, &[]);
        let binary = env::var("AISH_LLAMA_BIN").unwrap_or_else(|_| self.config.runtime_command.clone());
        let max_tokens = self.config.max_tokens.clamp(8, 64);
        let context_size = self.config.context_size.clamp(512, 2048);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let stdout_path = self.config.aish_home.join("cache").join(format!("runtime-{stamp}.out"));
        let stderr_path = self.config.aish_home.join("cache").join(format!("runtime-{stamp}.err"));
        fs::create_dir_all(self.config.aish_home.join("cache")).map_err(|err| err.to_string())?;
        let stdout_file = File::create(&stdout_path).map_err(|err| err.to_string())?;
        let stderr_file = File::create(&stderr_path).map_err(|err| err.to_string())?;

        eprintln!("Thinking...");

        let mut child = Command::new(&binary)
            .arg("-m")
            .arg(&self.config.model_path)
            .arg("-p")
            .arg(prompt)
            .arg("-n")
            .arg(max_tokens.to_string())
            .arg("--ctx-size")
            .arg(context_size.to_string())
            .arg("--temp")
            .arg(self.config.temperature.to_string())
            .arg("-t")
            .arg(self.config.threads.to_string())
            .arg("--no-display-prompt")
            .arg("--log-disable")
            .arg("--simple-io")
            .arg("--no-warmup")
            .arg("--single-turn")
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout_file))
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|err| {
                format!(
                    "failed to start AiSH model runtime: {err}"
                )
            })?;

        let timeout = Duration::from_secs(self.config.generation_timeout_seconds.max(5));
        let started = Instant::now();
        loop {
            if child.try_wait().map_err(|err| err.to_string())?.is_some() {
                break;
            }

            if started.elapsed() >= timeout {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&stdout_path);
                let _ = fs::remove_file(&stderr_path);
                return Err(format!(
                    "AiSH model took longer than {} seconds. Try again, or lower max_tokens in ~/.aish/config.toml.",
                    timeout.as_secs()
                ));
            }

            thread::sleep(Duration::from_millis(100));
        }

        let status = child.wait().map_err(|err| {
            format!("failed to wait for AiSH model runtime: {err}")
        })?;

        let stdout = read_bounded(&stdout_path)?;
        let stderr = read_bounded(&stderr_path)?;
        let _ = fs::remove_file(&stdout_path);
        let _ = fs::remove_file(&stderr_path);

        if !status.success() {
            return Err(format!("AiSH model runtime failed: {}", concise_error(&stderr)));
        }

        let command = clean_model_output(&stdout);
        if command.is_empty() {
            let fallback = mock_generate(shell, request);
            if !fallback.to_ascii_lowercase().contains("configure") {
                return Ok(fallback);
            }
            return Err("AiSH could not produce a clean command. Try a more specific request.".to_string());
        }

        Ok(command)
    }
}

fn read_bounded(path: &std::path::Path) -> Result<String, String> {
    let file = File::open(path).map_err(|err| err.to_string())?;
    let mut output = String::new();
    file.take(1_000_000)
        .read_to_string(&mut output)
        .map_err(|err| err.to_string())?;
    Ok(output)
}

fn concise_error(stderr: &str) -> String {
    let trimmed = stderr.trim();
    if trimmed.is_empty() {
        return "unknown runtime error".to_string();
    }

    trimmed
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or(trimmed)
        .trim()
        .to_string()
}

fn mock_generate(shell: ShellTarget, request: &str) -> String {
    let lower = request.to_ascii_lowercase();

    match shell {
        ShellTarget::PowerShell => mock_powershell(&lower),
        ShellTarget::Cmd => mock_cmd(&lower),
        ShellTarget::Bash | ShellTarget::Zsh => mock_bash(&lower),
    }
}

fn mock_bash(request: &str) -> String {
    if request.contains("python") && request.contains("file") && request.contains("recursive") {
        "find . -type f -name \"*.py\"".to_string()
    } else if request.contains("largest") && request.contains("file") {
        "find . -type f -exec du -h {} + | sort -hr | head -20".to_string()
    } else if request.contains("modified") && request.contains("24 hour") {
        "find . -type f -mtime -1".to_string()
    } else if request.contains("sha256") || request.contains("hash") {
        "sha256sum <file>".to_string()
    } else if request.contains("process") && request.contains("memory") {
        "ps aux --sort=-%mem | head -20".to_string()
    } else if request.contains("ssh") && (request.contains("check") || request.contains("status")) {
        "service ssh status".to_string()
    } else if request.contains("current directory") || request.contains("where am i") {
        "pwd".to_string()
    } else if request.contains("list") {
        "ls -la".to_string()
    } else {
        "echo \"Configure llama.cpp runtime to generate real commands\"".to_string()
    }
}

fn mock_powershell(request: &str) -> String {
    if request.contains("log") && request.contains("recursive") {
        "Get-ChildItem -Path . -Recurse -Filter *.log".to_string()
    } else if request.contains("python") && request.contains("file") {
        "Get-ChildItem -Path . -Recurse -Filter *.py".to_string()
    } else if request.contains("process") && request.contains("memory") {
        "Get-Process | Sort-Object WorkingSet -Descending | Select-Object -First 20".to_string()
    } else if request.contains("list") {
        "Get-ChildItem -Force".to_string()
    } else {
        "Write-Output \"Configure llama.cpp runtime to generate real commands\"".to_string()
    }
}

fn mock_cmd(request: &str) -> String {
    if request.contains("python") && request.contains("file") {
        "dir /s /b *.py".to_string()
    } else if request.contains("list") {
        "dir".to_string()
    } else {
        "echo Configure llama.cpp runtime to generate real commands".to_string()
    }
}
