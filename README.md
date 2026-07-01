# AiSH

AiSH is an AI-native interactive shell. You describe a task in natural language; AiSH generates the shell command internally, checks its risk, executes it, and returns the result.

Generated commands and routine safety metadata are hidden during normal use.

## Interaction

```text
$ aish
AiSH
Describe what you want to do. Type :help for settings or exit to quit.
aish> find all Python files in this folder
./src/main.py
./tests/test_main.py
```

AiSH has two output modes:

- `raw` prints the command's stdout and stderr unchanged.
- `natural` captures stdout and stderr, asks the local model to summarize them, and preserves exact paths, file names, identifiers, numbers, URLs, and errors.

Switch modes inside AiSH:

```text
:output raw
:output natural
```

Or choose the mode when starting it:

```bash
aish --raw-output
aish --natural-output
aish -n "show the five largest files here"
```

Natural-language output can be the default in `~/.aish/config.toml`:

```toml
[output]
natural_language_output = true
```

## Request Flow

```text
natural-language request
        |
        v
local model generates an internal command
        |
        v
safety classifier
        |
        +-- blocked/high risk --> refuse or request confirmation
        |
        v
selected system shell executes it
        |
        +-- raw mode ---------> stdout/stderr
        |
        +-- natural mode -----> local model summary
```

Safe commands execute without showing the generated command. Risky commands require confirmation and describe the risk without exposing the command. `--no-exec` remains available only as a development/debug preview.

## Run Locally

The default development backend is `mock`, so the UI can be tested without a model:

```bash
cargo run
cargo run -- --natural-output "list files"
```

Run tests:

```bash
cargo test
```

Use the real local model:

```bash
scripts/download-model.sh
AISH_RUNTIME=llama.cpp AISH_LLAMA_BIN=llama-cli cargo run
```

The default model location is `~/.aish/models/aish.gguf`.

## Bundle and Install

AiSH itself compiles to one native executable:

```bash
chmod +x scripts/install-local.sh
scripts/install-local.sh
```

This builds a release binary and installs it at `~/.local/bin/aish`. Add that directory to `PATH`, then start the shell with:

```bash
aish
```

The executable is the orchestration layer. Real AI inference also needs:

- `llama-cli` available on `PATH`, or selected with `AISH_LLAMA_BIN`
- the GGUF model at `~/.aish/models/aish.gguf`

For a single distributable environment containing AiSH and `llama-cli`, use Docker:

```bash
docker build -t aish .
docker run --rm -it -v aish-data:/home/aish/.aish aish
```

The Docker image contains the `aish` binary and `llama-cli`; the model is downloaded into the persistent `aish-data` volume on first use.

## Source Structure

```text
src/main.rs       CLI entry point and help
src/app.rs        interactive loop, output modes, safety and orchestration
src/runtime.rs    command generation and natural-language summarization
src/prompt.rs     command and summarization prompts/output cleaning
src/shell.rs      Bash/Zsh/PowerShell/CMD execution and output capture
src/safety.rs     command risk classification
src/config.rs     ~/.aish configuration
src/history.rs    local request/command history
src/model.rs      GGUF model setup
scripts/          local installer and model downloader
Dockerfile        bundled Linux runtime image
```

## Safety

The model never decides execution policy. AiSH classifies every internal command as safe, risky, high-risk, or blocked. Risky operations require confirmation; extreme destructive operations remain blocked.
