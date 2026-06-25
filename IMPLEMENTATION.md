# AiSH Beta Implementation Plan

This document describes the implementation plan for AiSH Beta, an AI-native shell that converts natural language into shell commands and executes them through a controlled orchestration layer.

## 1. Project Goal

AiSH Beta is intended to become an installable shell-like binary named `aish`.

The model is responsible for generating commands. The `aish` binary is responsible for everything around the model: runtime management, safety checks, context, configuration, command preview, confirmation, execution, history, and future shell integration.

The goal is not to build only a chatbot in the terminal. The goal is to build an AI-native terminal layer that can eventually work like a real shell replacement.

## 2. Core Architecture

```text
User natural language input
    ↓
AiSH interactive shell layer
    ↓
Context and history manager
    ↓
Prompt builder
    ↓
GGUF model runtime
    ↓
Generated shell command
    ↓
Safety and risk classifier
    ↓
Execution policy
    ↓
Real system shell
```

AiSH should not directly trust model output. The orchestrator must inspect every generated command before execution.

## 3. Main Binary

The main installable binary should be:

```text
aish
```

The binary should support interactive mode first:

```text
aish
```

Later, it can support one-shot usage:

```text
aish "find all Python files recursively"
```

The binary should be written in Rust.

## 4. Language Choice

Recommended language:

```text
Rust
```

Reasons:

- Fast native binary
- Memory safe
- Good for shell-like tools
- Good TTY and process handling ecosystem
- Good cross-platform support
- Easier to ship than Python
- Safer than C/C++ for a security-sensitive shell layer

## 5. Local AiSH Directory

AiSH should create a local directory in the user's home folder:

```text
~/.aish/
```

Suggested structure:

```text
~/.aish/
  config.toml
  models/
    aish.gguf
  history.db
  logs/
  cache/
```

Purpose:

```text
config.toml  - user configuration
models/      - local GGUF model files
history.db   - command and natural language history
logs/        - optional local logs
cache/       - runtime cache and temporary files
```

## 6. Model Format

AiSH Beta should use the GGUF model as the main runtime model.

Primary runtime model:

```text
GGUF Q4_K_M
```

Reason:

- Best for local CPU inference
- Good for edge devices
- Works with llama.cpp-style runtimes
- Lower RAM usage than full precision models
- Suitable for Linux and macOS testing
- Better for packaging into a local shell binary

## 7. Hugging Face Repos

| Repo | Link | Purpose |
| --- | --- | --- |
| Live Demo Space | [s41r4j/aish-beta](https://huggingface.co/spaces/s41r4j/aish-beta) | Web demo UI for testing AiSH input to command output |
| GGUF Model | [s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200) | Main model for local runtime, Docker testing, edge devices, llama.cpp, and AiSH shell package |
| LoRA Adapter | [s41r4j/aish-qwen25-coder-1.5b-lora-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-lora-200) | Fine-tuned adapter for future retraining and development |
| Merged Full Model | [s41r4j/aish-qwen25-coder-1.5b-merged-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-merged-200) | Full standalone Transformers/PyTorch model for testing or conversion workflows |
| Dataset | [s41r4j/aish-shell-command-dataset](https://huggingface.co/datasets/s41r4j/aish-shell-command-dataset) | Training dataset backup and future improvement source |

## 8. Private Model Download

For now, the GGUF repo may be private. During Docker testing and local development, AiSH should use a `.env` file to load the Hugging Face token.

Example `.env`:

```env
HF_TOKEN=your_huggingface_token_here
AISH_MODEL_REPO=s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200
AISH_MODEL_FILE=aish.gguf
```

The `.env` file must not be committed to the repository.

Recommended `.gitignore` entry:

```gitignore
.env
~/.aish/
*.gguf
```

The installer or startup flow should:

```text
1. Check if ~/.aish/models/aish.gguf exists
2. If it exists, use it
3. If it does not exist, read HF_TOKEN from environment or .env
4. Download the GGUF model from the private Hugging Face repo
5. Store it under ~/.aish/models/
6. Start AiSH runtime
```

## 9. Docker Testing Environment

Initial development and testing should happen inside Docker, not on the main host system.

The Docker container should be Linux-based and non-privileged.

Important safety rule:

```text
Do not mount the host root filesystem into the container.
```

Safe Docker testing goals:

- Install the `aish` binary inside the container
- Create `~/.aish/`
- Download or mount the GGUF model
- Run interactive AiSH shell mode
- Test command generation
- Test safety checks
- Test sudo detection
- Test history/context
- Test command execution in a contained environment

Suggested Docker environment behavior:

```text
Docker Linux container
    ↓
aish binary
    ↓
~/.aish/
    ↓
GGUF model
    ↓
interactive shell
    ↓
safe/risky command execution testing
```

## 10. Interactive Shell Behavior

AiSH should start as an interactive shell-like environment.

Example:

```text
$ aish
AiSH Beta
aish> find all Python files here
find . -type f -name "*.py"
executing...
```

For safe commands, AiSH can execute automatically if enabled in config.

For risky commands, AiSH must ask for confirmation.

For blocked commands, AiSH should refuse or require manual mode.

## 11. Command Generation Flow

AiSH should build a structured prompt for the model.

Expected prompt shape:

```text
System:
You are AiSH, an AI native shell.

Your job:
- Convert the user's natural language request into the correct shell command or script.
- Support Bash, Linux, PowerShell, CMD, Git, Docker, Kubernetes, networking, and developer CLI tasks.
- Output only the command or script unless the user asks for explanation.
- Do not wrap commands in markdown.
- Prefer safe, minimal, correct commands.
- Respect the requested shell target.

User:
Shell target: bash
User request: Find all Python files recursively from the current folder.
```

Expected model output:

```text
find . -type f -name "*.py"
```

The orchestrator should clean and validate the output before execution.

## 12. Safety Layer

The safety layer is mandatory.

The model only generates commands. The orchestrator decides whether to execute them.

Suggested risk levels:

```text
safe
risky
high_risk
blocked
```

### Safe Commands

Examples:

```text
ls
pwd
find . -type f -name "*.py"
du -sh *
ps aux
git status
```

Behavior:

```text
Execute directly if auto_execute_safe = true
```

### Risky Commands

Examples:

```text
rm
mv with overwrite risk
chmod
chown
curl or wget downloads
package installs
recursive modifications
service restarts
docker prune
```

Behavior:

```text
Show command
Explain risk shortly
Ask for confirmation
Execute only after approval
```

### High-Risk Commands

Examples:

```text
sudo
su
doas
disk operations
partition commands
system service changes
firewall changes
permission changes
commands touching /etc, /usr, /bin, /sbin, /var
```

Behavior:

```text
Show stronger confirmation
Require explicit approval
Use real system sudo for password prompt
Never store the sudo password
```

### Blocked Commands

Examples:

```text
rm -rf /
mkfs
dd to disk devices
fork bombs
credential stealing
secret dumping
destructive commands targeting system root
commands that intentionally damage the system
```

Behavior:

```text
Refuse by default
Allow only manual override in a future advanced mode, not in MVP
```

## 13. Sudo Handling

AiSH should not implement its own sudo password system.

Correct behavior:

```text
1. Detect sudo/su/doas in generated command
2. Mark command as high_risk
3. Show stronger confirmation
4. If approved, execute through the real system shell
5. Let the OS sudo prompt ask for the password
6. Let the OS manage sudo credential caching
```

AiSH must not:

```text
- Store sudo passwords
- Cache sudo passwords itself
- Ask for sudo password inside its own custom prompt
- Log sudo passwords
```

AiSH can rely on normal sudo behavior, where the OS may cache sudo authentication for a short session depending on system configuration.

## 14. Config File

Suggested config path:

```text
~/.aish/config.toml
```

Suggested config:

```toml
[aish]
shell = "bash"
model_path = "~/.aish/models/aish.gguf"
auto_execute_safe = true
confirm_risky = true
block_extreme_risk = true

[context]
enabled = true
context_commands = 10
suggest_after_seconds = 5

[safety]
confirm_sudo = true
confirm_delete = true
confirm_overwrite = true
confirm_network_download = true
block_root_delete = true
block_disk_format = true

[logging]
enabled = false
log_commands = false
log_model_prompts = false

[runtime]
backend = "llama.cpp"
threads = 4
context_size = 4096
temperature = 0.0
max_tokens = 256
```

## 15. Context Feature

AiSH should maintain recent context from previous commands.

Example:

```text
context_commands = 10
```

This means AiSH can use the last 10 natural language requests, generated commands, and command results as context.

The context system should help with:

- Better follow-up commands
- Natural language next-step suggestions
- Understanding current working directory behavior
- Improving command continuity

Example:

```text
User: show all log files
AiSH: find . -type f -name "*.log"

User waits for 5 seconds

AiSH suggestion:
You found log files. You may want to inspect the largest ones or search for errors.
```

Context should be configurable and possible to disable.

## 16. Idle Suggestions

AiSH can suggest next actions after the user is idle for a configured time.

Config:

```toml
[context]
suggest_after_seconds = 5
```

Behavior:

```text
1. User runs a command
2. AiSH stores command in history/context
3. User waits
4. AiSH generates a short natural language suggestion
5. AiSH does not execute anything automatically from suggestions
```

Suggestions should be natural language, not automatically executed commands.

## 17. Execution Engine

AiSH should execute commands through the real system shell.

Linux/macOS default:

```text
/bin/bash -lc "<command>"
```

Later macOS can support:

```text
/bin/zsh -lc "<command>"
```

Windows later:

```text
powershell.exe -Command "<command>"
cmd.exe /C "<command>"
```

For MVP, focus on Linux inside Docker using Bash.

## 18. TTY and Shell Features

Because AiSH aims to become a real shell-like layer, it must eventually handle:

- TTY input/output
- Ctrl+C
- Ctrl+D
- Signals
- Exit codes
- Current working directory
- Environment variables
- Pipes
- Redirection
- Interactive commands
- Long-running commands
- History navigation
- Basic autocomplete
- Job control where possible

Some shell behavior can be delegated to the real shell, but AiSH must still manage the interactive interface cleanly.

## 19. MVP Plan

### MVP 1: Docker Linux Interactive Shell

Scope:

```text
Rust aish binary
Docker Linux testing
interactive shell mode
~/.aish config
.env-based HF token loading
GGUF model download/check
GGUF runtime integration
Bash command generation
command output cleaning
safety engine
safe command auto-execute
risky command confirmation
sudo detection and strong confirmation
blocked command refusal
history storage
context from previous N commands
idle suggestions
```

Goal:

```text
AiSH works as an interactive AI-native shell inside Docker.
```

### MVP 2: Real Shell Replacement Path

Scope:

```text
macOS support
Linux host support
better TTY/job control
autocomplete
aliases
history sync
installer scripts
chsh/login-shell support
Windows PowerShell/CMD support
PowerShell command generation improvements
CMD command generation improvements
```

Goal:

```text
AiSH can move from Docker testing into a real installable shell-like tool.
```

## 20. Windows Strategy

AiSH does not need to be rebuilt from scratch for Windows.

The correct Windows direction is:

```text
Same AiSH core
Different platform adapter
```

The Rust core/orchestrator should stay shared across all platforms:

- model manager
- GGUF runtime integration
- prompt builder
- safety engine
- risk classification
- config loading
- history
- context
- command preview
- confirmation logic

Rust supports Windows targets, so the same codebase can produce Linux, macOS, and Windows binaries.

The Windows-specific work should live in a small platform adapter layer.

### Platform Adapter Split

```text
Linux/macOS:
  bash/zsh executor
  ~/.aish/
  sudo/su/doas detection

Windows:
  PowerShell/CMD executor
  %USERPROFILE%\.aish\ or %APPDATA%\AiSH\
  admin/UAC-aware high-risk handling
```

### Windows Executor

On Windows, AiSH should execute generated commands through the selected Windows shell target.

PowerShell:

```text
powershell.exe -Command "<command>"
```

CMD:

```text
cmd.exe /C "<command>"
```

Later, PowerShell Core can also be supported:

```text
pwsh.exe -Command "<command>"
```

The model prompt must include the selected shell target so the generated command matches the correct syntax.

Example:

```text
Shell target: powershell
User request: Find all log files in the current folder recursively.
```

Expected output:

```powershell
Get-ChildItem -Path . -Recurse -Filter *.log
```

### Windows Local Directory

Windows should not use Unix-only assumptions such as `~/.aish/` internally.

Recommended Windows storage paths:

```text
%USERPROFILE%\.aish\
```

or:

```text
%APPDATA%\AiSH\
```

Suggested Windows structure:

```text
%APPDATA%\AiSH\
  config.toml
  models\
    aish.gguf
  history.db
  logs\
  cache\
```

The implementation should use platform-aware path resolution instead of hard-coded paths.

### Windows Admin and UAC Handling

Windows does not use `sudo` as the normal elevation model.

AiSH should treat admin/elevation-sensitive commands as high risk and rely on normal Windows permissions and UAC behavior.

High-risk Windows examples:

```text
Commands requiring Administrator privileges
Registry modifications
Service changes
Firewall changes
System directory changes
Disk and partition operations
Credential or secret access
Recursive destructive file operations
```

AiSH should not implement its own Windows credential prompt.

Correct Windows behavior:

```text
1. Detect admin/elevation-sensitive command patterns
2. Mark command as high_risk
3. Show stronger confirmation
4. Execute through the selected Windows shell only after approval
5. Let Windows permissions or UAC handle elevation
6. Never store Windows credentials
```

### Windows Terminal Integration

Windows Terminal can host shells such as PowerShell and Command Prompt.

Later, AiSH can be packaged as its own Windows Terminal profile instead of replacing the full terminal stack.

Future Windows Terminal profile direction:

```text
Windows Terminal
    ↓
AiSH profile
    ↓
aish.exe
    ↓
PowerShell/CMD adapter
```

This keeps AiSH compatible with the normal Windows terminal ecosystem while still giving users an AI-native shell entrypoint.

### Windows GGUF Runtime

The GGUF model strategy should remain the same on Windows.

AiSH should reuse the same GGUF Q4_K_M model where possible:

```text
%APPDATA%\AiSH\models\aish.gguf
```

The runtime backend should support Windows builds through llama.cpp or a compatible GGUF runtime.

### Cross-Platform Build Direction

Build once architecturally, but compile and package per platform.

```text
Shared Rust core
    ↓
Platform adapter
    ↓
Platform binary/package

Linux:   aish + bash/zsh adapter
macOS:   aish + zsh/bash adapter
Windows: aish.exe + PowerShell/CMD adapter
```

MVP should still start with Linux inside Docker, but the code structure should avoid Linux-only assumptions so Windows support can be added without rewriting the product.

## 21. chsh Support

`chsh` means change shell.

On Linux and macOS, `chsh` is used to change the user's default login shell.

Example:

```text
chsh -s /path/to/aish
```

AiSH should not enable `chsh` support until the interactive shell behavior is stable. A broken login shell can make the user environment difficult to use.

## 22. Installer Direction

Initial installer behavior:

```text
1. Install aish binary
2. Create ~/.aish/
3. Create default config.toml
4. Check for GGUF model
5. If missing, download using HF_TOKEN
6. Start AiSH in interactive mode
```

Future installer targets:

```text
Linux install script
macOS install script
Homebrew formula
Debian package
RPM package
Windows installer
```

## 23. Logging and Privacy

AiSH should keep logs optional and local.

Default recommendation:

```text
logging.enabled = false
```

If logging is enabled, avoid storing secrets.

Do not log:

```text
sudo passwords
tokens
private keys
.env contents
SSH credentials
API keys
```

## 24. Repository Safety

The project repository should not include:

```text
.env
HF_TOKEN
GGUF model binaries
sudo passwords
user history
local logs
```

Recommended `.gitignore`:

```gitignore
.env
.env.*
*.gguf
~/.aish/
.aish/
target/
history.db
logs/
cache/
```

## 25. Final Implementation Direction

The correct implementation direction is:

```text
Build AiSH as a Rust-based interactive shell binary.
Use the GGUF Q4_K_M model as the local runtime model.
Use ~/.aish/ for config, model, history, logs, and cache.
Use Docker Linux as the first safe testing environment.
Use .env with HF_TOKEN during private model download testing.
Keep the model as command generator only.
Keep safety, sudo handling, execution, context, and confirmation in the orchestrator.
Auto-execute safe commands only.
Require confirmation for risky commands.
Require stronger confirmation for sudo/high-risk commands.
Block extreme destructive commands.
Keep Linux, macOS, and Windows on the same shared Rust core.
Implement platform-specific shell executors, storage paths, and elevation handling through adapters.
Add chsh and full shell replacement behavior only after stable interactive shell behavior.
```
