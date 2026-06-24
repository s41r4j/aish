# AiSH Model Training Plan

This document defines the model strategy for AiSH, the standalone intelligent terminal app.

The goal is not to train a large chatbot-style model first. The correct first model is a small on-device command ranker that improves local shell completion.

## Model Strategy

AiSH should use a staged model approach.

```text
v1: no model required
v2: tiny local command ranker
v3: optional command generator
```

## v1: Rule-Based Completion Engine

Before training any model, AiSH should implement deterministic completions.

Sources:

```text
- shell history
- current typed prefix
- current working directory
- package.json scripts
- Makefile targets
- Docker Compose services
- Git branches
- Cargo.toml
- pyproject.toml
- Gradle tasks
- common shell commands
```

Example:

```json
{
  "prefix": "npm",
  "cwd": "/projects/app",
  "detected_files": ["package.json", "vite.config.ts"],
  "candidates": [
    "npm run dev",
    "npm run build",
    "npm test"
  ]
}
```

This version should be fast enough to run on every keystroke.

Target latency:

```text
History/project suggestions: < 30 ms
Dropdown suggestions:        < 50 ms
AI-generated suggestions:    manual trigger only
```

## v2: Tiny Command Ranker

The first trainable model should rank candidate commands, not generate them from scratch.

Input:

```json
{
  "shell": "powershell",
  "os": "windows",
  "prefix": "npm",
  "cwd_type": "node_vite_project",
  "recent_commands": ["npm install", "npm run dev"],
  "candidates": [
    "npm run dev",
    "npm run build",
    "npm test",
    "npm install"
  ]
}
```

Output:

```json
[
  {"command": "npm run dev", "score": 0.94},
  {"command": "npm run build", "score": 0.42},
  {"command": "npm test", "score": 0.30},
  {"command": "npm install", "score": 0.18}
]
```

## Training Data

Training examples can come from:

```text
1. Local shell history
2. Accepted AiSH suggestions
3. Rejected AiSH suggestions
4. Successful commands
5. Failed commands
6. Public command examples
7. CLI documentation examples
8. Synthetic command/context pairs
```

Local personalization should stay on-device.

## Training Sample Format

```json
{
  "context": {
    "shell": "bash",
    "os": "linux",
    "prefix": "git che",
    "cwd_type": "git_repo",
    "git_branch": "main",
    "recent_commands": [
      "git status",
      "git branch",
      "git checkout feature-login"
    ]
  },
  "positive": "git checkout feature-login",
  "negatives": [
    "git checkout main",
    "git cherry-pick",
    "git checkout -- ."
  ]
}
```

## Feature Inputs

The model should receive compact features, not raw huge prompts.

Useful features:

```text
- typed prefix
- shell name
- OS
- current directory name
- project type
- detected files
- recent commands
- command frequency
- command recency
- last exit code
- Git branch
- package manager
- available scripts/tasks
```

## Model Type

Recommended options:

```text
Option A: Lightweight neural ranker
- small Transformer encoder or MLP over engineered features
- exported to ONNX
- fast inference

Option B: Gradient boosting / linear ranker
- simpler and very fast
- easier to train from limited data
- less flexible than neural ranking

Option C: Tiny command language model
- only for v3
- should not be the default model
```

Recommended first choice:

```text
Rule-based candidate generator + tiny ONNX ranker
```

## Model Runtime

Primary runtime:

```text
ONNX Runtime
```

Reasons:

```text
- works across Windows, macOS, and Linux
- good for small local models
- easier to call from Rust/C++/C#/Python ecosystems
- suitable for a fast command ranker
```

Optional future runtime:

```text
GGUF / llama.cpp
```

Use this only if AiSH later adds a small local command generator.

## Training Pipeline

```text
1. Collect local command events
2. Normalize shell-specific syntax
3. Detect project context
4. Generate candidate commands
5. Mark accepted commands as positives
6. Sample rejected or unused candidates as negatives
7. Train ranker
8. Export to ONNX
9. Quantize if needed
10. Evaluate latency and ranking quality
11. Ship model with AiSH
12. Continue local personalization on-device
```

## Event Logging Schema

AiSH should store local events in SQLite.

```sql
CREATE TABLE command_events (
  id TEXT PRIMARY KEY,
  timestamp INTEGER NOT NULL,
  shell TEXT NOT NULL,
  os TEXT NOT NULL,
  cwd_hash TEXT NOT NULL,
  project_type TEXT,
  typed_prefix TEXT,
  command TEXT NOT NULL,
  source TEXT NOT NULL,
  accepted INTEGER NOT NULL,
  exit_code INTEGER,
  duration_ms INTEGER
);
```

Suggested values for `source`:

```text
manual
history_suggestion
project_completion
ai_generated
```

## Evaluation Metrics

Track:

```text
- top-1 accuracy
- top-3 accuracy
- accepted suggestion rate
- rejected suggestion rate
- average suggestion latency
- dangerous suggestion rate
- per-shell quality
- per-project-type quality
```

Targets:

```text
Top-1 accuracy:              60%+ for repeated workflows
Top-3 accuracy:              80%+ for repeated workflows
History suggestion latency:  < 30 ms
Ranker inference latency:    < 10 ms
Dangerous silent suggestions: 0
```

## Safety Classifier

AiSH needs a deterministic safety layer before any suggestion is shown or accepted.

High-risk command patterns:

```text
rm -rf
sudo rm
rmdir /s
del /s /q
git reset --hard
docker system prune
kubectl delete
npm publish
chmod -R 777
drop database
format
```

Policy:

```text
- never auto-accept dangerous commands
- require confirmation for destructive commands
- show explanation before execution
- prefer safer alternatives where possible
```

## v3: Optional Command Generator

Once the ranker works well, AiSH can add a small command-generation model.

Use cases:

```text
- user types natural language
- user asks how to do a shell task
- user requests a command explanation
- user wants command alternatives
```

Example:

```text
User: find all large files over 500MB
AiSH: find . -type f -size +500M
```

This should be manually triggered with a shortcut such as:

```text
Ctrl + Space
```

AI generation should not run on every keystroke.

## Final Recommendation

Do not start by training a full LLM.

Start with:

```text
1. deterministic completion engine
2. local command history scorer
3. tiny ONNX ranker
4. optional command generator later
```

This gives AiSH the right balance of speed, privacy, safety, and usefulness.
