![image](https://github.com/user-attachments/assets/a2a45042-95b2-43fc-8fa0-7201b23707a9)

# AiSH Beta

AiSH Beta is an AI-native shell interface that converts natural language requests into shell commands.

Instead of remembering exact command syntax, the user can describe what they want to do, and AiSH generates the matching command for the selected shell target.

## What AiSH Does

AiSH helps users interact with the terminal using natural language.

Example:

```text
Find all Python files recursively from the current folder
```

AiSH can turn that into a shell command such as:

```bash
find . -type f -name "*.py"
```

The goal is to make terminal usage faster, simpler, and more accessible while still keeping command execution under user control.

## Supported Shell Targets

AiSH Beta is designed to support:

- Bash / Linux shell commands
- PowerShell commands
- CMD commands

The main focus is command generation for common terminal workflows, development tasks, file operations, system inspection, networking, and automation.

## Core Idea

AiSH is not a traditional chatbot.

It is a shell-first interface where the input is natural language and the output is a command or script that can be reviewed and executed.

The expected flow is:

```text
User request
    ↓
AiSH command generation
    ↓
Command preview
    ↓
User review
    ↓
Execution
```

## Key Features

- Natural language to shell command generation
- Bash and Linux command support
- PowerShell command support
- CMD command support
- Local and edge-friendly model usage
- Command preview before execution
- Useful for developer, system, and terminal workflows
- Designed for lightweight shell integration

## Example Requests

```text
Show the 20 largest files in this directory recursively
```

```text
Find all files modified in the last 24 hours
```

```text
Create a tar.gz backup of the current folder
```

```text
List running processes sorted by memory usage
```

```text
Calculate the SHA256 hash of a file
```

## Safety Model

AiSH should generate safe, minimal, and clear commands.

Commands should be shown to the user before execution, especially when they can modify, delete, overwrite, move, download, install, or execute files.

AiSH should prefer review-first behavior over blind execution.

## Project Status

AiSH Beta is currently focused on local command generation and shell package development.

The current model is suitable for basic Bash and Linux workflows, with early-stage support for PowerShell and CMD workflows.

## Current MVP

This repository now contains the first Rust implementation of the `aish` binary.

The current build includes:

- Interactive shell mode with the `aish>` prompt
- One-shot command generation mode
- Bash, Zsh, PowerShell, and CMD target selection
- Local AiSH config directory creation
- Prompt builder for AiSH command generation
- GGUF/llama.cpp runtime adapter
- Mock runtime for development without a downloaded model
- Safety classification: safe, risky, high-risk, blocked
- Command preview before execution
- Confirmation for risky and high-risk commands
- Blocked destructive command refusal
- Real shell execution through the selected shell adapter
- Local command history and context storage

## Local Development

If Rust is installed:

```bash
cargo run -- --no-exec "find all Python files recursively"
```

Interactive mode:

```bash
cargo run
```

Docker smoke test:

```bash
docker build -t aish-beta .
docker run --rm -it aish-beta
```

The default development runtime is `mock`, so the CLI can be tested before the GGUF model and llama.cpp runtime are installed.

To use the real GGUF runtime:

```bash
scripts/download-model.sh
AISH_RUNTIME=llama.cpp AISH_LLAMA_BIN=llama-cli cargo run
```

The model is expected at:

```text
~/.aish/models/aish.gguf
```

## Intended Use

AiSH Beta is intended for:

- Terminal productivity
- Developer command generation
- File and folder operations
- Shell workflow automation
- System inspection commands
- Networking and CLI tasks
- Local edge runtime experiments

## Current Limitations

- PowerShell and CMD support are still improving
- Complex multi-step automation may require review
- Generated commands should be checked before execution
- Destructive commands should require explicit confirmation
- AiSH Beta should not execute risky commands without user approval

## Command Behavior

By default, AiSH should output only the command or script unless the user asks for an explanation.

The command should match the selected shell target and stay as simple as possible.

## Summary

AiSH Beta is an AI-native shell that lets users control the terminal through natural language.

It aims to become a practical replacement layer for traditional command entry by generating shell commands from plain user requests.
