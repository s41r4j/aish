mod app;
mod config;
mod history;
mod model;
mod paths;
mod prompt;
mod runtime;
mod safety;
mod shell;

use app::{App, CliOptions};
use config::Config;
use std::env;

fn main() {
    if let Err(error) = run() {
        eprintln!("aish: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = CliOptions::parse(&args)?;

    if options.show_help {
        print_help();
        return Ok(());
    }

    if options.show_version {
        println!("aish {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let config = Config::load_or_create()?;
    let mut app = App::new(config, options)?;
    app.run()
}

fn print_help() {
    println!(
        "AiSH Beta\n\
\n\
Usage:\n\
  aish                         Start interactive shell mode\n\
  aish \"find all Python files\"  Generate and handle one command\n\
\n\
Options:\n\
  --shell <bash|zsh|powershell|cmd>  Select target shell\n\
  --no-exec                         Preview command without execution\n\
  --yes                             Skip confirmation for non-blocked commands\n\
  --help                            Show this help\n\
  --version                         Show version\n\
\n\
Environment:\n\
  HF_TOKEN=...                       Token used for the first private model download"
    );
}
