# AiSH

**AiSH** is a cross-platform intelligent terminal app built around three live-switchable modes: normal terminal usage, history-based completion, and local AI-generated command assistance.

AiSH is not just a shell plugin and it is not a chatbot CLI. It is a standalone terminal application that runs real shells inside its own UI and adds a smart input layer on top.

## Core Idea

```text
AiSH = terminal app + real shell runtime + smart completion layer
```

AiSH should work across:

```text
Windows: PowerShell, cmd, Git Bash
macOS:   Zsh, Bash
Linux:   Bash, Zsh, Fish
```

The app owns the terminal UI, suggestion rendering, ghost text, dropdowns, shortcuts, command cards, and AI mode. The actual commands still execute through the user's selected shell.

## Modes

### 1. Normal Mode

Plain terminal behavior.

```text
No prediction
No AI generation
No smart completion
Just type and run commands normally
```

This mode exists so AiSH always remains trustworthy and non-invasive.

### 2. History Mode

Suggests commands based on local usage.

AiSH uses:

```text
- recent commands
- frequent commands
- current working directory
- project type
- successful command patterns
- current typed prefix
```

Example:

```text
User types: npm
AiSH suggests: npm run dev
```

History Mode should be the default mode for the first release because it is fast, private, and useful without needing an AI model.

### 3. AI Mode

Generates command suggestions from user intent and local context.

Example:

```text
User types: find process using port 3000
AiSH suggests: netstat -ano | findstr :3000
```

AI Mode should be opt-in or shortcut-triggered, not constantly running on every keystroke.

## Suggested Shortcuts

```text
Ctrl + 1         Normal Mode
Ctrl + 2         History Mode
Ctrl + 3         AI Mode
Ctrl + Shift + M Cycle modes
Tab              Accept suggestion
Right Arrow      Accept ghost suggestion
Ctrl + Space     Open suggestions / ask AI
Esc              Dismiss suggestion
Alt + Enter      Explain selected command
```

## Product Architecture

```text
User types in AiSH terminal
        ↓
AiSH input layer captures current line
        ↓
Mode router
        ↓
Normal Mode  → pass through only
History Mode → local history scorer
AI Mode      → local model + project context
        ↓
Suggestion UI
        ↓
User accepts suggestion
        ↓
Command is sent to the real shell
```

## Technical Direction

Recommended stack:

```text
Desktop app:      Tauri + React
Terminal UI:      xterm.js
Native backend:   Rust
Windows shell:    ConPTY
macOS/Linux PTY:  Unix PTY
Local storage:    SQLite
Model runtime:    ONNX Runtime first, optional GGUF later
```

Recommended repo shape:

```text
aish/
├── apps/
│   └── desktop/
├── crates/
│   ├── aish-core/
│   ├── aish-pty/
│   ├── aish-history/
│   ├── aish-completion/
│   ├── aish-ai/
│   └── aish-context/
├── models/
├── docs/
│   └── MODEL_TRAINING_PLAN.md
└── README.md
```

## First Release Scope

The first version should focus on:

```text
- standalone terminal app
- running real shells inside AiSH
- mode switching
- command history storage
- history-based ghost suggestions
- dropdown suggestions
- project-aware completions for npm, git, docker, make, cargo, python
- safety checks for dangerous AI-generated commands
```

The first version does not need a full generative model. A deterministic completion engine plus a lightweight ranker is the safer path.

## Safety Rules

AiSH should not silently suggest destructive commands.

Examples of high-risk commands:

```text
rm -rf
del /s /q
git reset --hard
docker system prune
kubectl delete
npm publish
chmod -R 777
```

For risky commands, AiSH should require extra confirmation, show a warning, or avoid suggesting the command entirely.

## Project Status

This repository has been reset for the new AiSH direction.

Old plan:

```text
Python CLI that uses cloud/offline LLMs to generate and execute commands
```

New plan:

```text
Cross-platform standalone terminal app with Normal, History, and AI modes
```

See [`docs/MODEL_TRAINING_PLAN.md`](docs/MODEL_TRAINING_PLAN.md) for the model-training roadmap.
