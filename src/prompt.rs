use crate::history::HistoryEntry;
use crate::shell::ShellTarget;

pub fn build_command_prompt(shell: ShellTarget, request: &str, context: &[HistoryEntry]) -> String {
    let mut prompt = String::new();
    prompt.push_str("You are AiSH, an AI native shell command generator.\n");
    prompt.push_str("Return exactly one command or script for the requested shell.\n");
    prompt.push_str("Do not explain. Do not use markdown. Do not repeat the request.\n");
    prompt.push_str("Prefer safe, minimal, correct commands.\n\n");

    if !context.is_empty() {
        prompt.push_str("Recent context:\n");
        for entry in context {
            prompt.push_str("- User request: ");
            prompt.push_str(&entry.request);
            prompt.push_str("\n  Generated command: ");
            prompt.push_str(&entry.command);
            prompt.push('\n');
        }
        prompt.push('\n');
    }

    prompt.push_str("Shell target: ");
    prompt.push_str(shell.as_prompt_name());
    prompt.push('\n');
    prompt.push_str("User request: ");
    prompt.push_str(request);
    prompt.push_str("\nCommand:");

    prompt
}

pub fn clean_model_output(output: &str) -> String {
    let stripped = strip_ansi(output);
    let trimmed = stripped.trim();
    let without_fences = trimmed
        .strip_prefix("```bash")
        .or_else(|| trimmed.strip_prefix("```powershell"))
        .or_else(|| trimmed.strip_prefix("```cmd"))
        .or_else(|| trimmed.strip_prefix("```sh"))
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed);

    let cleaned = without_fences
        .trim()
        .strip_suffix("```")
        .unwrap_or(without_fences.trim())
        .trim()
        .to_string();

    first_command(&cleaned)
}

fn first_command(output: &str) -> String {
    let mut command_lines = Vec::new();

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            if !command_lines.is_empty() {
                break;
            }
            continue;
        }

        let lower = line.to_ascii_lowercase();
        if lower == "loading model..."
            || lower.starts_with("build ")
            || lower.starts_with("model ")
            || lower.starts_with("modalities ")
            || lower.starts_with("available commands:")
            || lower.starts_with("/exit")
            || lower.starts_with("/regen")
            || lower.starts_with("/clear")
            || lower.starts_with("/read")
            || lower.starts_with("/glob")
            || lower.starts_with("[ prompt:")
            || lower == "exiting..."
            || lower.starts_with("▄▄")
            || lower.starts_with("██")
            || lower.starts_with("▀▀")
            || line == ">"
            || line.starts_with("> ")
            || lower.starts_with("to ")
            || lower.starts_with("you can ")
            || lower.starts_with("this command ")
            || lower.starts_with("the command ")
            || lower.starts_with("here ")
            || lower.starts_with("you are aish")
            || lower.starts_with("return exactly")
            || lower.starts_with("do not ")
            || lower.starts_with("prefer safe")
            || lower.starts_with("recent context")
            || lower.starts_with("- user request:")
            || lower.starts_with("generated command:")
            || lower.starts_with("- generated command:")
        {
            if let Some(after_prompt) = line.strip_prefix("> ") {
                let after_prompt = after_prompt.trim();
                if looks_like_command(after_prompt) {
                    command_lines.push(after_prompt.to_string());
                    break;
                }
            }
            continue;
        }

        if lower.starts_with("command:")
            || lower.starts_with("shell target:")
            || lower.starts_with("user request:")
            || lower.starts_with("system:")
            || lower.starts_with("user:")
            || lower.starts_with("assistant:")
            || lower.contains("llama_")
            || lower.contains("llama.cpp")
        {
            let Some((_, after_colon)) = line.split_once(':') else {
                continue;
            };

            if lower.starts_with("command:") && !after_colon.trim().is_empty() {
                command_lines.push(after_colon.trim().to_string());
                break;
            }

            continue;
        }

        command_lines.push(line.to_string());

        if !line.ends_with('\\') && !line.ends_with('|') && !line.ends_with("&&") {
            break;
        }
    }

    command_lines.join("\n").trim().to_string()
}

fn looks_like_command(line: &str) -> bool {
    let first = line.split_whitespace().next().unwrap_or("");
    matches!(
        first,
        "ls"
            | "pwd"
            | "find"
            | "grep"
            | "awk"
            | "sed"
            | "cat"
            | "du"
            | "df"
            | "ps"
            | "ssh"
            | "systemctl"
            | "service"
            | "curl"
            | "wget"
            | "git"
            | "docker"
            | "kubectl"
            | "test"
            | "nc"
            | "ss"
            | "netstat"
            | "command"
            | "which"
            | "where"
    )
}

fn strip_ansi(value: &str) -> String {
    let mut output = String::new();
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::clean_model_output;

    #[test]
    fn removes_markdown_fence() {
        assert_eq!(clean_model_output("```bash\nls -la\n```"), "ls -la");
    }

    #[test]
    fn extracts_command_label() {
        assert_eq!(clean_model_output("Command: ssh -V\n\nDone"), "ssh -V");
    }
}
