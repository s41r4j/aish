use crate::config::Config;
use crate::history::{HistoryEntry, HistoryStore};
use crate::runtime::Runtime;
use crate::safety::{RiskLevel, SafetyEngine};
use crate::shell::{ShellExecutor, ShellTarget};
use std::io::{self, Write};

#[derive(Clone, Debug)]
pub struct CliOptions {
    pub request: Option<String>,
    pub shell: Option<ShellTarget>,
    pub no_exec: bool,
    pub assume_yes: bool,
    pub show_help: bool,
    pub show_version: bool,
}

impl CliOptions {
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self {
            request: None,
            shell: None,
            no_exec: false,
            assume_yes: false,
            show_help: false,
            show_version: false,
        };

        let mut request_parts = Vec::new();
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--help" | "-h" => options.show_help = true,
                "--version" | "-V" => options.show_version = true,
                "--no-exec" => options.no_exec = true,
                "--yes" | "-y" => options.assume_yes = true,
                "--shell" | "-s" => {
                    index += 1;
                    let value = args
                        .get(index)
                        .ok_or_else(|| "--shell requires a value".to_string())?;
                    options.shell = Some(
                        ShellTarget::parse(value)
                            .ok_or_else(|| format!("unsupported shell target '{value}'"))?,
                    );
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown option '{value}'"));
                }
                value => request_parts.push(value.to_string()),
            }
            index += 1;
        }

        if !request_parts.is_empty() {
            options.request = Some(request_parts.join(" "));
        }

        Ok(options)
    }
}

pub struct App {
    config: Config,
    runtime: Runtime,
    history: HistoryStore,
    options: CliOptions,
}

impl App {
    pub fn new(mut config: Config, options: CliOptions) -> Result<Self, String> {
        if let Some(shell) = options.shell {
            config.shell = shell;
        }

        let runtime = Runtime::new(config.clone());
        let history = HistoryStore::new(config.history_path());

        Ok(Self {
            config,
            runtime,
            history,
            options,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        if let Some(request) = self.options.request.clone() {
            return self.handle_request(&request).map(|_| ());
        }

        self.run_interactive()
    }

    fn run_interactive(&mut self) -> Result<(), String> {
        println!("AiSH Beta");
        println!("type natural language, ':help', or 'exit'");

        let stdin = io::stdin();
        loop {
            print!("aish> ");
            io::stdout().flush().map_err(|err| err.to_string())?;

            let mut line = String::new();
            let bytes = stdin.read_line(&mut line).map_err(|err| err.to_string())?;
            if bytes == 0 {
                println!();
                return Ok(());
            }

            let input = line.trim();
            if input.is_empty() {
                continue;
            }

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                return Ok(());
            }

            if input.starts_with(':') {
                self.handle_meta_command(input)?;
                continue;
            }

            if let Err(error) = self.handle_request(input) {
                eprintln!("{error}");
            }
        }
    }

    fn handle_meta_command(&mut self, input: &str) -> Result<(), String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.as_slice() {
            [":help"] => {
                println!(":shell <bash|zsh|powershell|cmd>  change shell target");
                println!(":noexec on|off                    preview without execution");
                println!(":config                           show config paths");
                println!(":history                          show recent generated commands");
                println!("exit                              quit");
            }
            [":config"] => {
                println!("shell: {}", self.config.shell);
                println!("model: {}", self.config.model_path.display());
                println!("model source: {}/{}", self.config.model_repo, self.config.model_file);
            }
            [":history"] => {
                for entry in self.history.recent(self.config.context_commands) {
                    println!("[{}] {} -> {}", entry.risk, entry.request, entry.command);
                }
            }
            [":shell", value] => {
                let shell = ShellTarget::parse(value)
                    .ok_or_else(|| format!("unsupported shell target '{value}'"))?;
                self.config.shell = shell;
                println!("shell={shell}");
            }
            [":noexec", value] => {
                self.options.no_exec = matches!(*value, "on" | "true" | "1" | "yes");
                println!("no_exec={}", self.options.no_exec);
            }
            _ => println!("unknown meta command. Try :help"),
        }

        Ok(())
    }

    fn handle_request(&mut self, request: &str) -> Result<Option<i32>, String> {
        let context = if self.config.context_enabled {
            self.history.recent(self.config.context_commands)
        } else {
            Vec::new()
        };

        let command = self
            .runtime
            .generate(self.config.shell, request, &context)
            .map(|command| command.trim().to_string())?;

        if command.is_empty() {
            return Err("runtime returned an empty command".to_string());
        }

        let assessment = SafetyEngine::assess(&command, self.config.shell);

        println!("{command}");
        println!("risk: {}", assessment.level.label());
        for reason in &assessment.reasons {
            println!("reason: {reason}");
        }

        if assessment.level == RiskLevel::Blocked && self.config.block_extreme_risk {
            println!("blocked: command was not executed");
            self.write_history(request, &command, assessment.level, None)?;
            return Ok(None);
        }

        if self.options.no_exec {
            self.write_history(request, &command, assessment.level, None)?;
            return Ok(None);
        }

        let should_execute = match assessment.level {
            RiskLevel::Safe => self.config.auto_execute_safe || self.options.assume_yes || ask("Execute? [y/N] ")?,
            RiskLevel::Risky => {
                if self.options.assume_yes || !self.config.confirm_risky {
                    true
                } else {
                    ask("Risky command. Execute? [y/N] ")?
                }
            }
            RiskLevel::HighRisk => {
                if self.options.assume_yes {
                    true
                } else {
                    ask("High-risk command. Type y to execute: ")?
                }
            }
            RiskLevel::Blocked => false,
        };

        if !should_execute {
            println!("not executed");
            self.write_history(request, &command, assessment.level, None)?;
            return Ok(None);
        }

        let executor = ShellExecutor::new(self.config.shell);
        let exit_code = executor.execute(&command)?;
        self.write_history(request, &command, assessment.level, Some(exit_code))?;
        Ok(Some(exit_code))
    }

    fn write_history(
        &self,
        request: &str,
        command: &str,
        risk: RiskLevel,
        exit_code: Option<i32>,
    ) -> Result<(), String> {
        let entry = HistoryEntry {
            request: request.to_string(),
            command: command.to_string(),
            risk: risk.label().to_string(),
            exit_code,
        };

        self.history.append(&entry)
    }
}

fn ask(prompt: &str) -> Result<bool, String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|err| err.to_string())?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|err| err.to_string())?;

    Ok(matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes"))
}
