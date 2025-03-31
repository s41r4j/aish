# AiSH - Artificially Intelligent Shell (v0.1 Beta)

![logo](/extras/logos/AiSH_logo_v1.png)

**AiSH** is an experimental command-line tool that blends AI smarts with shell functionality, letting you control your system through natural language or traditional commands.   
**Heads up: this is a beta release (v0.1), so it’s buggy!** Some features might not work as expected, and we’d really appreciate it if you’d report any issues on [GitHub Issues](https://github.com/s41r4j/aish/issues). Better yet, if you spot something broken or have a cool feature idea, send us a pull request to fix it or add it—every contribution helps AiSH grow into a seriously useful tool. We’re counting on you to pitch in as much as you can, whether it’s a tiny tweak or a big upgrade. Let’s make this awesome together!

<br><hr><br>

## 🚀 Installation

AiSH runs on **Python 3.11+**. Here’s how to set it up:

1. **Grab the Code**:
   ```bash
   git clone https://github.com/s41r4j/aish.git
   cd aish
   python3 -m venv .venv
   source .venv/bin/activate
   ```
   For activating your new venv (windows; above is for linux/unix based systems):
   - Run `./.venv/Scripts/Activate.ps1` via Powershell.
   - Or run `./.venv/Scripts/Activate.bat` via cmd.
   - Or on WSL/Linux, simply run source `./.venv/Scripts/activate`


2. **Install Dependencies**:
   ```bash
   pip install -r src/requirements.txt
   ```

3. **Launch AiSH**:
   ```bash
   python3 src/aish.py
   ```

4. **Config File**:
   The first time you run AiSH, it creates a config file at `~/.aishrc` (e.g., `/home/s41r4j/.aishrc` on Linux or `C:\Users\s41r4j\.aishrc` on Windows). Edit this YAML file to tweak settings like the prompt theme or AI backend—details are in the **Configuration** section below.

<br><hr><br>

## 💡 Usage

AiSH’s prompt accepts three types of commands:

- **Natural Language (NL)**: Just type what you want in plain English, like `list all files` or `check disk space`, and AiSH figures out the command for you.
- **Direct Shell Commands (`!`)**: Prefix with `!` to run raw shell commands, e.g., `!dir` (Windows) or `!ls` (Linux).
- **AiSH-Specific Commands (`/`)**: Use `/` for built-ins like `/help` (see options) or `/exit` (quit).

### Quick Examples
- NL: `show current time` → AiSH runs `date` or `time`.
- `!`: `!echo "Hi!"` → Prints "Hi!".
- `/`: `/prompt mood` → Switches to a fun emoji-based prompt.

<br><hr><br>

## ⚙️ Configuration

AiSH creates a configuration file called .aishrc in your home directory (~/.aishrc on Linux/macOS or %USERPROFILE%\.aishrc on Windows) the first time you run it. You can tweak it to customize your experience.
Settings You Can Change

`prev_cmds_limit`: How many past commands to include in AI prompts (default: 5).  
`prompt_theme`:  
- `default`: Plain "AiSH> " prompt.  
- `pwd`: Shows your current directory (e.g., "AiSH /home/user> ").  
- `mood`: Adds an emoji (😊 for success, 😞 for failure).
  
`mode`: Set to "online" for AI features (default) or "offline" (not yet supported).  
`online.current`: Pick your AI backend (e.g., "groq").  

The `.aishrc` file (e.g., `/home/s41r4j/.aishrc`) lets you customize AiSH. EG:
```yaml
aish:
  mode: online
  prev_cmds_limit: 5
  prompt_theme: default
offline:
  ollama_model: ''
online:
  apis:
    gemini:
      api_key: ''
      model: 'gemini-2.0-flash'
    groq:
      api_key: ''
      model: 'deepseek-r1-distill-llama-70b'
  current: groq
  fallback: gemini
```

> **NOTE: Currently AiSH only supports online modes with two apis -> [Gorq](https://console.groq.com/keys) (this works well) and [Gemini](https://aistudio.google.com/app/apikey); Ollama also works but has potential bugs**

<br><hr><br>

## 🌱 Future Prospects

AiSH aims to become a proper shell—like `bash`, `sh`, or `fish` on Linux, or `cmd` and PowerShell on Windows—but supercharged with AI. We’re focused on making it a standalone, powerful command-line environment. That’s the dream, and we’re sticking to it!

---

## 📢 Call for Feedback, Support, and Contributions

AiSH v0.1 is rough around the edges, and we need your help to polish it up! Found a bug? Please report it on [GitHub Issues](https://github.com/s41r4j/aish/issues). Got a feature idea or a fix? Submit a pull request—we’d love your code! Even small contributions make a big difference in turning AiSH into a tool you’ll want to use every day. Share your thoughts, spread the word, and join us in building something great. What do you say—ready to help?

<br><hr><br>



That’s AiSH v0.1 (beta)! Dive in, play around, and let us know how we can make it better. Happy hacking!
