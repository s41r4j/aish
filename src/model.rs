use crate::config::Config;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn ensure_model(config: &Config, assume_yes: bool) -> Result<(), String> {
    if config.model_path.exists() {
        return Ok(());
    }

    let model_dir = config
        .model_path
        .parent()
        .unwrap_or(&config.model_download_dir);
    fs::create_dir_all(model_dir).map_err(|err| {
        format!(
            "could not create model directory {}: {err}",
            model_dir.display()
        )
    })?;

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        config.model_repo, config.model_file
    );
    let temp_path = config.model_path.with_extension("download");

    eprintln!("AiSH model not found: {}", config.model_path.display());
    eprintln!("Model source: {}/{}", config.model_repo, config.model_file);
    eprintln!("This is a one-time download into your AiSH home directory.");

    if config.confirm_network_download && !assume_yes && !ask_download()? {
        return Err("model download cancelled".to_string());
    }

    eprintln!("Preparing AiSH model.");

    let mut command = Command::new("curl");
    command.arg("-L").arg("--fail").arg("--progress-bar");
    if let Ok(token) = env::var("HF_TOKEN") {
        if !token.trim().is_empty() {
            command
                .arg("-H")
                .arg(format!("Authorization: Bearer {token}"));
        }
    }
    let status = command
        .arg("-o")
        .arg(&temp_path)
        .arg(&url)
        .stdin(Stdio::null())
        .status()
        .map_err(|err| format!("could not start model download: {err}"))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "model download failed from {}/{}",
            config.model_repo, config.model_file
        ));
    }

    fs::rename(&temp_path, &config.model_path).map_err(|err| {
        format!(
            "download finished but could not save model to {}: {err}",
            config.model_path.display()
        )
    })?;

    eprintln!("AiSH model is ready.");
    Ok(())
}

fn ask_download() -> Result<bool, String> {
    eprint!("Download model now? [y/N] ");
    io::stderr().flush().map_err(|err| err.to_string())?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|err| err.to_string())?;

    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}
