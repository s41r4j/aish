# AiSH Model Assets and Runtime Details

This file tracks the AiSH Beta model artifacts, runtime assets, and related Hugging Face resources.

## AiSH Hugging Face Repos

| Repo | Link | Purpose |
| --- | --- | --- |
| Live Demo Space | [s41r4j/aish-beta](https://huggingface.co/spaces/s41r4j/aish-beta) | Web demo UI for testing AiSH input to command output |
| GGUF Model | [s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200) | Best for local and edge runtime, llama.cpp, and Ollama-style usage |
| LoRA Adapter | [s41r4j/aish-qwen25-coder-1.5b-lora-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-lora-200) | Fine-tuned adapter for retraining and development |
| Merged Full Model | [s41r4j/aish-qwen25-coder-1.5b-merged-200](https://huggingface.co/s41r4j/aish-qwen25-coder-1.5b-merged-200) | Full standalone Transformers/PyTorch model |
| Dataset | [s41r4j/aish-shell-command-dataset](https://huggingface.co/datasets/s41r4j/aish-shell-command-dataset) | Training dataset backup |

## Model Details

```text
Project: AiSH / AI Shell
Product name: AiSH Beta
Base model: unsloth/Qwen2.5-Coder-1.5B-Instruct
Fine-tuning method: LoRA / QLoRA
Training steps: 200
Primary task: Natural language to shell command
Main shells: Bash, PowerShell, CMD
Dataset size: approximately 14,683 rows
Dataset mix:
- Bash/Linux/macOS commands: approximately 14,663 rows
- PowerShell seed examples: approximately 20 rows, oversampled during training
Final edge format: GGUF Q4_K_M
Demo runtime: Hugging Face Space + FastAPI + llama.cpp
```

## Artifact Roles

```text
GGUF Q4_K_M model
Use for the AiSH app/runtime, local CPU inference, edge devices, and shell package integration.

LoRA adapter
Use for future fine-tuning, continued training, experiments, and model updates.

Merged full model
Use for Transformers/PyTorch inference, browser testing, model export, and conversion workflows.

Dataset
Use for reproducibility, retraining, evaluation, and improving Bash, PowerShell, and CMD support.

Live demo Space
Use for public or private testing through a simple web interface.
```

## Which Asset Matters Most

```text
For AiSH app/runtime: GGUF repo
For retraining: LoRA + dataset repos
For browser or Transformers testing: merged model repo
For public demo: Space repo
```

## Runtime Wrapper Direction

AiSH Beta should use the GGUF Q4_K_M model as the main runtime model for the shell package.

Recommended local runtime stack:

```text
AiSH CLI wrapper
    ↓
User natural language request
    ↓
Shell target selection
    ↓
llama.cpp or compatible GGUF runtime
    ↓
Generated command
    ↓
Command preview
    ↓
User confirmation
    ↓
Shell execution
```

## Expected Prompt Format

AiSH should send the model a system prompt and a user message containing the target shell and the user request.

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

## Runtime Output Rule

The model should return only the command or script.

Example:

```text
find . -type f -name "*.py"
```

## Recommended Package Behavior

The AiSH shell package should:

- Accept a natural language request
- Detect or accept the target shell
- Generate a command using the GGUF runtime
- Show the generated command before execution
- Ask for confirmation before running risky commands
- Execute only after user approval
- Provide an option to copy the command without running it
- Keep logs optional and local

## Safety Notes

AiSH Beta should treat the following command types as risky:

- Delete operations
- Overwrite operations
- Recursive file changes
- Permission changes
- Network downloads
- Package installs
- Remote execution
- Disk formatting or partitioning
- Credential or secret access
- Commands using sudo or administrator permissions

Risky commands should require explicit confirmation before execution.

## Private Repo Note

If any Hugging Face repos are private, the user must be logged into Hugging Face with access to view or download them.
