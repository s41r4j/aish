use crate::config::Config;
use crate::context::SystemContext;
use crate::history::HistoryEntry;
use crate::model::ensure_model;
use crate::prompt::{
    build_command_prompt, build_summary_prompt, clean_model_output, clean_summary_output,
};
use crate::shell::ShellTarget;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub struct Runtime {
    config: Config,
    assume_yes: bool,
}

impl Runtime {
    pub fn new(config: Config, assume_yes: bool) -> Self {
        Self { config, assume_yes }
    }

    pub fn prepare(&self) -> Result<(), String> {
        if self.uses_llama_runtime() {
            ensure_model(&self.config, self.assume_yes)?;
            self.resolve_llama_binary()?;
        }

        Ok(())
    }

    pub fn generate(
        &self,
        shell: ShellTarget,
        request: &str,
        context: &[HistoryEntry],
    ) -> Result<String, String> {
        let backend = self.runtime_backend();

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

    pub fn summarize(&self, stdout: &str, stderr: &str, exit_code: i32) -> Result<String, String> {
        let backend = self.runtime_backend();

        if backend.eq_ignore_ascii_case("mock") {
            return Ok(mock_summarize(stdout, stderr, exit_code));
        }

        if backend.eq_ignore_ascii_case("llama.cpp") || backend.eq_ignore_ascii_case("llama") {
            ensure_model(&self.config, self.assume_yes)?;
            let prompt = build_summary_prompt(stdout, stderr, exit_code);
            let output = self.run_llama(&prompt, self.config.max_tokens.clamp(64, 512))?;
            let summary = clean_summary_output(&output);
            let summary = if summary.is_empty() {
                mock_summarize(stdout, stderr, exit_code)
            } else {
                summary
            };
            return Ok(preserve_exact_paths(summary, stdout, stderr));
        }

        Err(format!(
            "unsupported runtime backend '{backend}'. Use 'mock' or 'llama.cpp'"
        ))
    }

    fn generate_with_llama(
        &self,
        shell: ShellTarget,
        request: &str,
        context: &[HistoryEntry],
    ) -> Result<String, String> {
        ensure_model(&self.config, self.assume_yes)?;

        let system_context = SystemContext::capture();
        let prompt = build_command_prompt(shell, request, context, &system_context);
        let max_tokens = self.config.max_tokens.clamp(8, 64);
        let stdout = self.run_llama(&prompt, max_tokens)?;
        let command = clean_model_output(&stdout);
        if command.is_empty() {
            let fallback = mock_generate(shell, request);
            if !fallback.to_ascii_lowercase().contains("configure") {
                return Ok(fallback);
            }
            return Err(
                "AiSH could not produce a clean command. Try a more specific request.".to_string(),
            );
        }

        Ok(command)
    }

    fn run_llama(&self, prompt: &str, max_tokens: usize) -> Result<String, String> {
        let binary = self.resolve_llama_binary()?;
        let context_size = self.config.context_size.clamp(512, 8192);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let stdout_path = self
            .config
            .aish_home
            .join("cache")
            .join(format!("runtime-{stamp}.out"));
        let stderr_path = self
            .config
            .aish_home
            .join("cache")
            .join(format!("runtime-{stamp}.err"));
        fs::create_dir_all(self.config.aish_home.join("cache")).map_err(|err| err.to_string())?;
        let stdout_file = File::create(&stdout_path).map_err(|err| err.to_string())?;
        let stderr_file = File::create(&stderr_path).map_err(|err| err.to_string())?;

        let mut command = Command::new(&binary);
        apply_runtime_library_path(&mut command, &binary);

        let mut child = command
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
            .map_err(|err| format!("failed to start AiSH model runtime: {err}"))?;

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

        let status = child
            .wait()
            .map_err(|err| format!("failed to wait for AiSH model runtime: {err}"))?;

        let stdout = read_bounded(&stdout_path)?;
        let stderr = read_bounded(&stderr_path)?;
        let _ = fs::remove_file(&stdout_path);
        let _ = fs::remove_file(&stderr_path);

        if !status.success() {
            return Err(format!(
                "AiSH model runtime failed: {}",
                concise_error(&stderr)
            ));
        }

        Ok(stdout)
    }

    fn runtime_backend(&self) -> String {
        env::var("AISH_RUNTIME").unwrap_or_else(|_| self.config.runtime_backend.clone())
    }

    fn uses_llama_runtime(&self) -> bool {
        let backend = self.runtime_backend();
        backend.eq_ignore_ascii_case("llama.cpp") || backend.eq_ignore_ascii_case("llama")
    }

    fn resolve_llama_binary(&self) -> Result<PathBuf, String> {
        if let Ok(value) = env::var("AISH_LLAMA_BIN") {
            if !value.trim().is_empty() {
                return Ok(PathBuf::from(value));
            }
        }

        let configured = PathBuf::from(&self.config.runtime_command);
        if configured.components().count() > 1 {
            return Ok(configured);
        }

        for candidate in bundled_llama_candidates() {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        Ok(configured)
    }
}

fn bundled_llama_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    let binary_name = llama_binary_name();

    if let Ok(current_exe) = env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            candidates.push(exe_dir.join(&binary_name));
            candidates.push(exe_dir.join("runtime").join(&binary_name));
            if let Some(package_dir) = exe_dir.parent() {
                candidates.push(package_dir.join("runtime").join(&binary_name));
                candidates.push(package_dir.join("libexec").join("aish").join(&binary_name));
            }
        }
    }

    #[cfg(windows)]
    {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            candidates.push(
                PathBuf::from(local_app_data)
                    .join("AiSH")
                    .join("runtime")
                    .join(&binary_name),
            );
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = env::var("HOME") {
            candidates.push(
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("aish")
                    .join("runtime")
                    .join(&binary_name),
            );
        }
    }

    candidates
}

fn llama_binary_name() -> String {
    if cfg!(windows) {
        "llama-cli.exe".to_string()
    } else {
        "llama-cli".to_string()
    }
}

fn apply_runtime_library_path(command: &mut Command, binary: &Path) {
    if cfg!(windows) {
        return;
    }

    let Some(runtime_dir) = binary.parent() else {
        return;
    };

    let key = if cfg!(target_os = "macos") {
        "DYLD_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    };

    let mut paths = vec![runtime_dir.to_path_buf()];
    if let Some(existing) = env::var_os(key) {
        paths.extend(env::split_paths(&existing));
    }

    if let Ok(value) = env::join_paths(paths) {
        command.env(key, value);
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

fn mock_summarize(stdout: &str, stderr: &str, exit_code: i32) -> String {
    let stdout = stdout.trim();
    let stderr = stderr.trim();

    if stdout.is_empty() && stderr.is_empty() {
        return if exit_code == 0 {
            "The command completed successfully and produced no output.".to_string()
        } else {
            format!("The command failed with exit code {exit_code} and produced no output.")
        };
    }

    let content = if !stderr.is_empty() && !stdout.is_empty() {
        format!("{stdout}\n{stderr}")
    } else if !stderr.is_empty() {
        stderr.to_string()
    } else {
        stdout.to_string()
    };
    let lines: Vec<&str> = content.lines().collect();
    let result = if lines.len() <= 6 {
        content
    } else {
        format!(
            "{}\n… {} more lines …\n{}",
            lines[..4].join("\n"),
            lines.len() - 5,
            lines[lines.len() - 1]
        )
    };

    if exit_code == 0 {
        format!("The command completed successfully. {result}")
    } else {
        format!("The command failed with exit code {exit_code}. {result}")
    }
}

fn preserve_exact_paths(mut summary: String, stdout: &str, stderr: &str) -> String {
    let combined = format!("{stdout}\n{stderr}");
    let mut missing_paths = Vec::new();

    for raw_token in combined.split_whitespace() {
        let token = raw_token.trim_matches(|character: char| {
            matches!(
                character,
                '"' | '\'' | '`' | ',' | ';' | ':' | '(' | ')' | '[' | ']'
            )
        });
        let is_path = token.starts_with('/')
            || token.starts_with("./")
            || token.starts_with("../")
            || token.starts_with("~/");
        if is_path && !summary.contains(token) && !missing_paths.iter().any(|path| *path == token) {
            missing_paths.push(token);
        }
        if missing_paths.len() == 20 {
            break;
        }
    }

    if !missing_paths.is_empty() {
        if !summary.ends_with('.') && !summary.ends_with('!') && !summary.ends_with('?') {
            summary.push('.');
        }
        summary.push_str(" Exact path");
        if missing_paths.len() > 1 {
            summary.push('s');
        }
        summary.push_str(": ");
        summary.push_str(&missing_paths.join(", "));
        summary.push('.');
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::preserve_exact_paths;

    #[test]
    fn appends_a_path_omitted_by_the_model() {
        assert_eq!(
            preserve_exact_paths(
                "The current directory was shown.".to_string(),
                "/home/aish\n",
                ""
            ),
            "The current directory was shown. Exact path: /home/aish."
        );
    }

    #[test]
    fn does_not_repeat_a_preserved_path() {
        assert_eq!(
            preserve_exact_paths(
                "The current directory is /home/aish.".to_string(),
                "/home/aish\n",
                ""
            ),
            "The current directory is /home/aish."
        );
    }
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
