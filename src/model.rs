use crate::config::Config;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

pub fn ensure_model(config: &Config) -> Result<(), String> {
    if config.model_path.exists() {
        return Ok(());
    }

    fs::create_dir_all(&config.model_download_dir).map_err(|err| {
        format!(
            "could not create model directory {}: {err}",
            config.model_download_dir.display()
        )
    })?;

    let token = env::var("HF_TOKEN").map_err(|_| {
        format!(
            "AiSH model is not installed yet. Set HF_TOKEN, then start AiSH again. The model will be saved to {}",
            config.model_path.display()
        )
    })?;

    if token.trim().is_empty() {
        return Err("HF_TOKEN is empty. Set a valid Hugging Face token and start AiSH again.".to_string());
    }

    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}",
        config.model_repo, config.model_file
    );
    let temp_path = config.model_path.with_extension("download");

    eprintln!("Preparing AiSH model. This is a one-time download.");

    let status = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--progress-bar")
        .arg("-H")
        .arg(format!("Authorization: Bearer {token}"))
        .arg("-o")
        .arg(&temp_path)
        .arg(&url)
        .stdin(Stdio::null())
        .status()
        .map_err(|err| format!("could not start model download: {err}"))?;

    if !status.success() {
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "model download failed. Check HF_TOKEN access to {}",
            config.model_repo
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
