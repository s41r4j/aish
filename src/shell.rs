use std::fmt;
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellTarget {
    Bash,
    Zsh,
    PowerShell,
    Cmd,
}

impl ShellTarget {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "bash" | "linux" | "sh" => Some(Self::Bash),
            "zsh" | "macos" | "mac" => Some(Self::Zsh),
            "powershell" | "power-shell" | "ps" => Some(Self::PowerShell),
            "cmd" | "command-prompt" => Some(Self::Cmd),
            _ => None,
        }
    }

    pub fn default_for_platform() -> Self {
        #[cfg(windows)]
        {
            Self::PowerShell
        }

        #[cfg(target_os = "macos")]
        {
            Self::Zsh
        }

        #[cfg(all(not(windows), not(target_os = "macos")))]
        {
            Self::Bash
        }
    }

    pub fn as_config_value(&self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::PowerShell => "powershell",
            Self::Cmd => "cmd",
        }
    }

    pub fn as_prompt_name(&self) -> &'static str {
        self.as_config_value()
    }
}

impl fmt::Display for ShellTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_config_value())
    }
}

pub struct ShellExecutor {
    target: ShellTarget,
}

pub struct ExecutionOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ShellExecutor {
    pub fn new(target: ShellTarget) -> Self {
        Self { target }
    }

    pub fn execute(&self, command: &str) -> Result<ExecutionOutput, String> {
        let mut process = match self.target {
            ShellTarget::Bash => {
                let mut command_process = Command::new("/bin/bash");
                command_process.arg("-lc").arg(command);
                command_process
            }
            ShellTarget::Zsh => {
                let mut command_process = Command::new("/bin/zsh");
                command_process.arg("-lc").arg(command);
                command_process
            }
            ShellTarget::PowerShell => {
                let mut command_process = Command::new("powershell.exe");
                command_process.arg("-Command").arg(command);
                command_process
            }
            ShellTarget::Cmd => {
                let mut command_process = Command::new("cmd.exe");
                command_process.arg("/C").arg(command);
                command_process
            }
        };

        let output = process
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|err| {
                format!(
                    "failed to execute command through {target}: {err}",
                    target = self.target
                )
            })?;

        Ok(ExecutionOutput {
            exit_code: output.status.code().unwrap_or(1),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}
