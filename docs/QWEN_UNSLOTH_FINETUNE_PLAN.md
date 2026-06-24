# AiSH Qwen + Unsloth Fine-Tuning Plan

This document defines the first practical model-training track for AiSH.

## Decision

AiSH will fine-tune a small Qwen-family model with Unsloth.

After checking the currently visible official Qwen model pages, the practical first target should be:

```text
1. Qwen/Qwen3-0.6B
2. Qwen/Qwen3-1.7B if 0.6B is underwhelming
3. Qwen/Qwen2.5-Coder-0.5B-Instruct as a coding-specialized comparison
4. Qwen/Qwen2.5-Coder-1.5B-Instruct if the 0.5B coder model is too weak
```

Notes:

```text
- I did not find a verified official Qwen3.5-0.8B model ID.
- Qwen3-0.6B is the closest verified sub-1B Qwen3 target.
- Qwen3-1.7B is the closest verified step-up model near the original 2B fallback idea.
- Qwen2.5-Coder-0.5B/1.5B are useful comparison baselines because AiSH is command/code-adjacent.
```

The first goal is not to create a general chatbot. The model should be specialized for terminal command assistance.

## AiSH Model Role

The model should help with:

```text
- command generation from short user intent
- command completion from partial input
- command explanation
- command alternative suggestions
- shell-aware command translation
- OS-aware command translation
- structured command JSON for the AiSH runtime
```

The model should not own execution.

Execution flow:

```text
User input
  ↓
AiSH UI
  ↓
History/project completion layer
  ↓
Fine-tuned Qwen model, only when needed
  ↓
Command candidate
  ↓
AiSH runtime safety detector
  ↓
User confirmation if risky
  ↓
Shell execution
```

## Guardrail Position

AiSH should not train the model to behave like a restricted assistant that refuses normal terminal tasks.

Instead:

```text
Model layer:      command-focused, direct, minimal refusal behavior
Runtime layer:    detects risky/destructive commands
UX layer:         asks for confirmation before risky execution
```

The model can output commands, explanations, alternatives, and structured JSON. AiSH decides whether the command needs confirmation before execution.

## Why Start Small

A sub-1B model is the correct first experiment because AiSH needs low latency and on-device execution.

Target behavior:

```text
- fast enough for interactive use
- small enough for laptops
- good at common shell workflows
- useful for command generation when manually triggered
```

If Qwen3-0.6B cannot produce reliable commands after fine-tuning, move to Qwen3-1.7B.

## Training Method

Use Unsloth with LoRA or QLoRA.

Recommended first setup:

```text
Method:           QLoRA
Base precision:   4-bit
LoRA rank:        16 or 32
Sequence length:  512 to 2048
Batching:         gradient accumulation
Training style:   supervised fine-tuning
```

The first training run should optimize output format and command correctness, not broad reasoning.

## Recommended Model Experiments

### Experiment A: Qwen3-0.6B

```text
Base model:       Qwen/Qwen3-0.6B
Trainer:          Unsloth
Method:           QLoRA
Dataset size:     5k to 20k high-quality examples
Eval set:         500 to 1k examples
Goal:             fast local command generation
```

This is the primary first experiment.

### Experiment B: Qwen3-1.7B

```text
Base model:       Qwen/Qwen3-1.7B
Trainer:          Unsloth
Method:           QLoRA
Dataset size:     20k to 50k examples
Eval set:         1k to 2k examples
Goal:             better command generation and shell reasoning
```

Use this if Qwen3-0.6B is too weak.

### Experiment C: Qwen2.5-Coder-0.5B-Instruct

```text
Base model:       Qwen/Qwen2.5-Coder-0.5B-Instruct
Trainer:          Unsloth
Method:           QLoRA
Dataset size:     5k to 20k examples
Eval set:         500 to 1k examples
Goal:             compare code-specialized small model behavior
```

Use this as a small coder-specialized baseline.

### Experiment D: Qwen2.5-Coder-1.5B-Instruct

```text
Base model:       Qwen/Qwen2.5-Coder-1.5B-Instruct
Trainer:          Unsloth
Method:           QLoRA
Dataset size:     20k to 50k examples
Eval set:         1k to 2k examples
Goal:             stronger coder-specialized command model
```

Use this if the 0.5B coder model is too weak.

## Dataset Format

Use instruction-style JSONL.

Example:

```json
{"instruction":"Suggest a command for the user's terminal task.","input":"OS: windows\nShell: powershell\nCWD type: node_vite_project\nUser input: start this app","output":"npm run dev"}
```

Example with explanation:

```json
{"instruction":"Return a command and a short explanation.","input":"OS: macos\nShell: zsh\nCWD type: git_repo\nUser input: show changed files","output":"Command: git status --short\nExplanation: Shows modified, staged, and untracked files in a compact format."}
```

Example with alternatives:

```json
{"instruction":"Suggest command alternatives.","input":"OS: linux\nShell: bash\nUser input: list files with details","output":"1. ls -la\n2. find . -maxdepth 1 -type f\n3. tree -a -L 1"}
```

Example with structured output:

```json
{"instruction":"Return structured command JSON for AiSH.","input":"OS: windows\nShell: powershell\nCWD type: unknown\nUser input: find process using port 3000","output":"{\"command\":\"netstat -ano | findstr :3000\",\"confidence\":0.86,\"requires_confirmation\":false,\"reason\":\"Lists processes bound to port 3000 on Windows.\"}"}
```

## Dataset Sources

Use a mixture of:

```text
- generated command-intent pairs
- package manager examples
- git examples
- docker examples
- python tooling examples
- node/npm/pnpm/yarn examples
- cargo examples
- make examples
- powershell examples
- cmd examples
- bash/zsh/fish examples
- accepted AiSH suggestions later
- rejected AiSH suggestions later
```

## Dataset Buckets

Create separate buckets so evaluation is easier.

```text
1. command_generation
2. command_completion
3. command_explanation
4. command_alternatives
5. shell_translation
6. os_translation
7. project_aware_commands
8. structured_json_output
9. risky_command_labeling
```

The risky-command bucket is for labeling and explanation quality. Execution safety remains inside AiSH runtime, not inside the model.

## Output Formats

AiSH should train predictable output formats.

### Command only

```text
npm run dev
```

### Command with metadata

```json
{
  "command": "npm run dev",
  "confidence": 0.86,
  "requires_confirmation": false,
  "reason": "Detected a Node project with a dev script."
}
```

The structured JSON format is better for app integration. The command-only format is better for raw latency experiments.

## Evaluation

Measure:

```text
- exact command match
- semantic command match
- top-3 useful suggestion rate
- invalid command rate
- wrong shell syntax rate
- wrong OS command rate
- structured JSON validity
- latency after quantization
- risky command flagging accuracy
```

Minimum acceptance targets for Qwen3-0.6B:

```text
Command validity:        85%+
Correct shell syntax:    85%+
JSON validity:           95%+
Useful top-3 rate:       75%+
Risky flag recall:       95%+
```

If it misses these after dataset cleanup and one retune, move to Qwen3-1.7B or the Qwen2.5-Coder comparison path.

## Runtime Safety

Runtime safety is outside the model.

AiSH should parse a generated command before execution and classify it as:

```text
safe
needs_confirmation
blocked_by_policy
```

For normal development commands, the command can be accepted quickly.

For risky commands, AiSH should show:

```text
- command
- reason it is risky
- affected path/service if detectable
- explicit confirmation prompt
```

The model should not be relied on as the final safety authority.

## Export Plan

After fine-tuning:

```text
1. Save LoRA adapter
2. Merge adapter into base model for release builds if needed
3. Quantize for local inference
4. Export GGUF for llama.cpp-style runtimes if generation is used locally
5. Keep ONNX or a smaller ranker path for ultra-fast ranking
```

AiSH can use two model paths:

```text
Fast path:  deterministic suggestions + ranker
AI path:    fine-tuned Qwen generator on manual trigger
```

## Final Recommendation

Start with Qwen/Qwen3-0.6B through Unsloth, not an unverified 0.8B model ID.

The correct first model milestone is:

```text
A small local command generator that is useful when triggered manually,
while AiSH runtime handles destructive-command confirmation before execution.
```
