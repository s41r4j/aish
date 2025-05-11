# AiSH: AI Shell Assistant

![image](https://github.com/user-attachments/assets/a2a45042-95b2-43fc-8fa0-7201b23707a9)


**AiSH** (Artificially Intelligent Shell) is an autonomous, AI-driven shell assistant that interprets user input, generates shell commands, and executes them on your real machine. It supports both online (cloud LLMs) and offline (Ollama) modes, with robust error handling, command history, and dynamic configuration.

<br>

## Features

- **AI-Powered Command Generation:** Converts natural language into shell commands using LLMs (Groq, Gemini, OpenRouter, Ollama).
- **Autonomous Task Processing:** Breaks down complex tasks into step-by-step shell commands and executes them.
- **Error Correction:** Uses AI to analyze and retry failed commands.
- **Interactive Shell:** Enhanced prompt with auto-completion, history, and customizable themes.
- **Secure API Key Storage:** API keys are encrypted using Fernet.
- **Cross-Platform:** Works on Linux, macOS, and Windows.


<br>

## Getting Started

### 1. Install Requirements

```sh
pip install -r requirements.txt
```

### 2. Configure API Keys

- On first run, `.aishrc` is created in your home directory.
- It is recommended to add your API keys and preferred models using the commands `/config api edit <api> key <value>` and `/config api edit <api> model <value>`.
- You can also add them directly in `.aishrc`, but above method is preferred.
- API keys are securely encrypted automatically.

### 3. Run AiSH

```sh
python src/aish.py
```


<br>

## Usage

### Shell Commands

- Enter natural language requests (e.g., `list files in home directory`).
- Use `/help` for a list of commands.

### Special Commands

| Command                | Description                                  |
|------------------------|----------------------------------------------|
| `/help` or `/h`        | Show help                                    |
| `/verbose` or `/v`     | Toggle verbose mode                          |
| `/config` or `/c`      | Configure APIs, history, etc.                |
| `/prompt [theme]`      | Change prompt theme (default, pwd, mood)     |
| `/exit` or `/e`        | Exit AiSH                                    |
| `!<cmd>`               | Execute raw shell command                    |

### Configuration

- Use `/config api current <api>` to set the active AI provider.
- Use `/config api edit <api> key <key>` to update API keys.
- Use `/config api edit <api> model <model>` to update models.
- Use `/config prev_cmds <n>` to set command history length in prompts.


<br>

## Security

- API keys are securely stored and never saved in plaintext.
- Your sensitive configuration remains protected at all times.


<br>

## Example Tasks

- `list files and folders`
- `write hello world python program`
- `how much disk space is left?`
- `is docker running?`


<br>

## Troubleshooting

- If you see errors about missing API keys, edit `.aishrc` and add your keys.
- For offline mode, ensure Ollama is running and the model is set in `.aishrc`.


<br>

For more details, see the code in the [src/](../src) directory.
