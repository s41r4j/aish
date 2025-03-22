# AiSH - AI-Driven Shell Assistant

    AiSH
  🌟 An intelligent shell assistant powered by AI

**AiSH** (AI Shell) is a modular, interactive command-line tool that leverages AI to execute single commands, answer questions, and perform autonomous multi-step tasks on your system. Built with Python, it integrates with the Groq API (with Ollama support planned) and features a fancy, emoji-enhanced UI with command history and verbose mode.

## Features
- **Single Commands**: Execute simple shell commands (e.g., `list home dir contents` → `ls ~`).
- **Autonomous Tasks**: Perform complex tasks step-by-step (e.g., `create a hello.txt with "hello world" written`).
- **Questions**: Answer queries without execution (e.g., `what is your model`).
- **Verbose Mode**: Display executed commands with `/verbose`.
- **Fancy UI**: Terminal-safe emojis (🌟, 🚀, ✅) and structured formatting.
- **History Navigation**: Use ↑↓ for previous commands, ←→ for editing, and auto-suggestions.
- **Modular Design**: Separate modules for AI queries, system info, and task management.

## Requirements
- Python 3.6+
- Dependencies (listed in `requirements.txt`):
  ```
  colorama==0.4.6
  psutil==5.9.8
  requests==2.31.0
  groq==0.5.0
  python-dotenv==1.0.1
  prompt_toolkit==3.0.47
  ```

## Setup
1. **Clone the Repository**:
   ```bash
   git clone <repository-url>
   cd aish
   ```

2. **Install Dependencies**:
   ```bash
   pip install -r requirements.txt
   ```

3. **Set Up Environment Variables**:
   - Create a `.env` file in the project root:
     ```bash
     GROQ_API_KEY=your_groq_api_key_here
     ```
   - (Optional) For Ollama (not yet active):
     ```
     OLLAMA_METHOD=cli
     OLLAMA_API_URL=http://localhost:11434/api/generate
     OLLAMA_MODEL=deepseek-r1:1.5b
     ```

4. **Run AiSH**:
   ```bash
   python3 aish.py
   ```

## Usage
Start AiSH and interact via the prompt:
```
=== 🌟 AiSH v0.1 ===
  💻 OS       : Linux 5.15.0-73-generic
  ⚙️ CPU      : 4 cores @ 12.5%
  📦 RAM      : 7923 MB total, 3456 MB free
Type '/help' for commands. Use ↑↓ for history, ←→ to edit, Ctrl+C to exit/stop.
🌟 AiSH>
```

### Commands
- **AiSH Commands**: Prefix with `/`
  - `/help`: Show available commands.
  - `/system`: Display system info.
  - `/verbose`: Toggle verbose mode (shows executed commands).
  - `/exit` or `/quit`: Exit AiSH.
- **Direct Shell Execution**: Prefix with `!`
  - `!ls`: Runs `ls` directly.
- **Normal Input**: AI interprets your intent
  - `list home dir contents`: Executes `ls ~`.
  - `what is your model`: Answers with text.
  - `create a hello.txt with "hello world" written`: Autonomous task.

### Examples
1. **Single Command**:
   ```
   🌟 AiSH> list home dir contents
   <home directory contents>
   ```

2. **Autonomous Task with Verbose**:
   ```
   🌟 AiSH> /verbose
   📢 Verbose mode enabled
   🌟 AiSH> create a hello.txt with "hello world" written
   🚀 Starting task: create a hello.txt with "hello world" written
   📢 Executing: echo "hello world" > hello.txt
   ✅ Task completed
   ```

3. **Question**:
   ```
   🌟 AiSH> what is your model
   ℹ️ I’m powered by deepseek-r1-distill-llama-70b via Groq
   ```

## Project Structure
```
.
├── ai_interface.py    # AI query logic (Groq + Ollama)
├── aish.py           # Main shell application
├── README.md         # This file
├── requirements.txt  # Dependencies
├── system_info.py    # System information utilities
├── task_manager.py   # Autonomous task processing
├── TASKS.txt         # Task notes (if applicable)
└── utils.py          # Command execution utilities
```

## Contributing
Feel free to fork, submit PRs, or report issues! Future plans include:
- Full Ollama integration.
- Configuration via `.aishrc`.
- Enhanced intent classification.

## License
MIT License - Free to use, modify, and distribute.

### **Notes**
- **Branding**: Used `AiSH` consistently as the logo/name.
- **Emojis**: Kept terminal-safe emojis (🌟, 🚀, etc.) from the code for a cohesive look.
- **Setup**: Included `.env` setup for Groq API key, with placeholders for Ollama.
- **Examples**: Mirrored the fancy UI output from your tests.
