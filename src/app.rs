use crate::config::Config;
use crate::context::SystemContext;
use crate::history::{HistoryEntry, HistoryStore};
use crate::runtime::Runtime;
use crate::safety::{RiskLevel, SafetyEngine};
use crate::shell::{ExecutionOutput, ShellExecutor, ShellTarget};
use std::env;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputMode {
    Raw,
    Natural,
}

#[derive(Clone, Debug)]
pub struct CliOptions {
    pub request: Option<String>,
    pub shell: Option<ShellTarget>,
    pub no_exec: bool,
    pub assume_yes: bool,
    pub output_mode: Option<OutputMode>,
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
            output_mode: None,
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
                "--natural-output" | "-n" => options.output_mode = Some(OutputMode::Natural),
                "--raw-output" => options.output_mode = Some(OutputMode::Raw),
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
        if let Some(mode) = options.output_mode {
            config.natural_language_output = mode == OutputMode::Natural;
        }

        let runtime = Runtime::new(config.clone(), options.assume_yes);
        let history = HistoryStore::new(config.history_path());

        Ok(Self {
            config,
            runtime,
            history,
            options,
        })
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.runtime.prepare()?;

        if let Some(request) = self.options.request.clone() {
            return self.handle_request(&request).map(|_| ());
        }

        self.run_interactive()
    }

    fn run_interactive(&mut self) -> Result<(), String> {
        println!("AiSH");
        println!("Describe what you want to do. Type / for AiSH commands or exit to quit.");
        self.print_context_summary();

        let stdin = io::stdin();
        loop {
            let context = SystemContext::capture();
            print!("{} aish> ", context.prompt_prefix(self.config.shell.as_prompt_name()));
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

            if is_exit(input) {
                return Ok(());
            }

            if input.starts_with('/') {
                if let Err(error) = self.handle_slash_command(input) {
                    eprintln!("{error}");
                }
                continue;
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
                print_slash_help();
            }
            [":config"] => {
                self.print_config();
            }
            [":history"] => {
                self.print_history();
            }
            [":shell", value] => {
                let shell = ShellTarget::parse(value)
                    .ok_or_else(|| format!("unsupported shell target '{value}'"))?;
                self.config.shell = shell;
                println!("shell={shell}");
            }
            [":output", "natural"] => {
                self.config.natural_language_output = true;
                println!("Natural-language output enabled.");
            }
            [":output", "raw"] => {
                self.config.natural_language_output = false;
                println!("Raw output enabled.");
            }
            _ => println!("unknown meta command. Try :help"),
        }

        Ok(())
    }

    fn handle_slash_command(&mut self, input: &str) -> Result<(), String> {
        let mut parts = input.split_whitespace();
        let command = parts.next().unwrap_or("/");
        let rest = parts.collect::<Vec<_>>().join(" ");

        match command {
            "/" | "/help" | "/commands" => print_slash_help(),
            "/exit" | "/quit" => {}
            "/context" | "/whoami" => self.print_context_summary(),
            "/config" => self.print_config(),
            "/history" => self.print_history(),
            "/pwd" => println!("{}", SystemContext::capture().cwd.display()),
            "/cd" => self.change_directory(&rest)?,
            "/shell" => self.change_shell(&rest)?,
            "/output" => self.change_output(&rest),
            "/clear" => print!("\x1b[2J\x1b[H"),
            _ => println!("unknown AiSH command. Type / to see commands."),
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

        if assessment.level == RiskLevel::Blocked && self.config.block_extreme_risk {
            println!("This request was blocked because it could cause extreme system damage.");
            self.write_history(request, &command, assessment.level, None)?;
            return Ok(None);
        }

        if self.options.no_exec {
            println!("{command}");
            self.write_history(request, &command, assessment.level, None)?;
            return Ok(None);
        }

        let should_execute = match assessment.level {
            RiskLevel::Safe => {
                self.config.auto_execute_safe || self.options.assume_yes || ask("Execute? [y/N] ")?
            }
            RiskLevel::Risky => {
                if self.options.assume_yes || !self.config.confirm_risky {
                    true
                } else {
                    describe_risk(&assessment.reasons);
                    ask("This action may modify your system. Continue? [y/N] ")?
                }
            }
            RiskLevel::HighRisk => {
                if self.options.assume_yes {
                    true
                } else {
                    describe_risk(&assessment.reasons);
                    ask("This is a high-risk action. Continue? [y/N] ")?
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
        let output = executor.execute(&command)?;
        self.present_output(&output)?;
        self.write_history(request, &command, assessment.level, Some(output.exit_code))?;
        Ok(Some(output.exit_code))
    }

    fn present_output(&self, output: &ExecutionOutput) -> Result<(), String> {
        if self.config.natural_language_output {
            let summary =
                self.runtime
                    .summarize(&output.stdout, &output.stderr, output.exit_code)?;
            println!("{summary}");
            return Ok(());
        }

        print!("{}", output.stdout);
        eprint!("{}", output.stderr);
        io::stdout().flush().map_err(|err| err.to_string())?;
        io::stderr().flush().map_err(|err| err.to_string())
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

    fn print_config(&self) {
        println!("shell: {}", self.config.shell);
        println!("model: {}", self.config.model_path.display());
        println!(
            "model source: {}/{}",
            self.config.model_repo, self.config.model_file
        );
        println!(
            "runtime: {} ({})",
            self.config.runtime_backend, self.config.runtime_command
        );
        println!(
            "output: {}",
            if self.config.natural_language_output {
                "natural"
            } else {
                "raw"
            }
        );
    }

    fn print_history(&self) {
        for entry in self.history.recent(self.config.context_commands) {
            let outcome = entry
                .exit_code
                .map(|code| format!("exit {code}"))
                .unwrap_or_else(|| "not executed".to_string());
            println!("[{}; {}] {}", entry.risk, outcome, entry.request);
        }
    }

    fn print_context_summary(&self) {
        let context = SystemContext::capture();
        for line in context.describe_lines() {
            println!("{line}");
        }
        println!("shell target: {}", self.config.shell);
        println!(
            "output mode: {}",
            if self.config.natural_language_output {
                "natural"
            } else {
                "raw"
            }
        );
    }

    fn change_directory(&self, value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            return Err("usage: /cd <path>".to_string());
        }

        let path = crate::paths::expand_user_path(value.trim());
        env::set_current_dir(&path)
            .map_err(|err| format!("could not change directory to {}: {err}", path.display()))?;
        println!("cwd={}", SystemContext::capture().cwd.display());
        Ok(())
    }

    fn change_shell(&mut self, value: &str) -> Result<(), String> {
        if value.trim().is_empty() {
            println!("shell={}", self.config.shell);
            return Ok(());
        }

        let shell = ShellTarget::parse(value)
            .ok_or_else(|| format!("unsupported shell target '{}'", value.trim()))?;
        self.config.shell = shell;
        println!("shell={shell}");
        Ok(())
    }

    fn change_output(&mut self, value: &str) {
        match value.trim() {
            "" => println!(
                "output={}",
                if self.config.natural_language_output {
                    "natural"
                } else {
                    "raw"
                }
            ),
            "natural" => {
                self.config.natural_language_output = true;
                println!("output=natural");
            }
            "raw" => {
                self.config.natural_language_output = false;
                println!("output=raw");
            }
            _ => println!("usage: /output raw|natural"),
        }
    }
}

fn print_slash_help() {
    println!("/                         show AiSH commands");
    println!("/context                  show user, host, OS, arch, shell, and cwd");
    println!("/pwd                      show current working directory");
    println!("/cd <path>                change AiSH working directory");
    println!("/shell [bash|zsh|powershell|cmd]  show or change shell target");
    println!("/output [raw|natural]     show or change output mode");
    println!("/config                   show model and runtime config");
    println!("/history                  show recent requests");
    println!("/clear                    clear the screen");
    println!("/exit                     quit AiSH");
}

fn is_exit(input: &str) -> bool {
    matches!(
        input.to_ascii_lowercase().as_str(),
        "exit" | "quit" | "/exit" | "/quit"
    )
}

fn describe_risk(reasons: &[String]) {
    for reason in reasons {
        println!("{reason}");
    }
}

fn ask(prompt: &str) -> Result<bool, String> {
    print!("{prompt}");
    io::stdout().flush().map_err(|err| err.to_string())?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|err| err.to_string())?;

    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
