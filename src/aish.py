from colorama import Fore, Style, init
from core.system_info import get_system_info
from core.utils import execute_command, encrypt_api_key, decrypt_api_key
from core.task_manager import process_autonomous_task
from core.ai_interface import query_ai
from core.prompt_factory import PromptFactory, format_history, clean_response
from core.config import config
from core.config import get_config_path
import re
import sys
import signal
from prompt_toolkit import PromptSession
from prompt_toolkit.history import FileHistory
from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
from prompt_toolkit.styles import Style as PromptStyle
from prompt_toolkit.completion import Completer, Completion
import os
from datetime import datetime
import yaml
import uuid

# Initialize colorama for cross-platform color support
init(autoreset=True)

# Global variables
last_error = None
task_running = False
verbose_mode = False
history = []  # Store conversation history

# Custom signal handler for Ctrl+C
def signal_handler(sig, frame):
    if task_running:
        print(f"{Fore.MAGENTA}🛑 Stopping task...{Style.RESET_ALL}")
        sys.exit(0)
    else:
        print(f"{Fore.MAGENTA}🛑 Exiting AiSH...{Style.RESET_ALL}")
        sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)

# Fancy prompt style
prompt_style = PromptStyle.from_dict({
    'prompt': 'fg:#00FF00 bold',  # Bright green, bold
})

class AiSHCompleter(Completer):
    def get_completions(self, document, complete_event):
        text = document.text_before_cursor.lstrip()
        words = text.split()
        # Top-level commands and aliases
        commands = ["/help", "/h", "/verbose", "/v", "/config", "/c", "/prompt", "/exit", "/e", "/quit"]
        # Always suggest top-level commands if at the start
        if not words or (text.startswith("/") and len(words) == 1):
            for cmd in commands:
                if cmd.lower().startswith(text.lower()):
                    yield Completion(cmd, start_position=-len(text))
            return
        # /config and /c subcommands
        if words[0] in ("/config", "/c"):
            # Suggest section after /config or /c
            if len(words) == 1 or (len(words) == 2 and not text.endswith(" ")):
                for sec in ["api", "prev_cmds"]:
                    if len(words) == 1 or sec.startswith(words[1].lower()):
                        yield Completion(sec, start_position=0 if len(words)==1 else -len(words[1]))
                return
            # /config api ...
            if words[1] == "api":
                # Suggest subcommands for api
                if len(words) == 2 or (len(words) == 3 and not text.endswith(" ")):
                    for opt in ["current", "fallback", "edit"]:
                        if len(words) == 2 or opt.startswith(words[2].lower()):
                            yield Completion(opt, start_position=0 if len(words)==2 else -len(words[2]))
                    return
                # /config api current|fallback ...
                if words[2] in ("current", "fallback"):
                    apis = list(config.get("online", {}).get("apis", {}).keys())
                    if len(words) == 3 or (len(words) == 4 and not text.endswith(" ")):
                        for api in apis:
                            if len(words) == 3 or api.startswith(words[3].lower()):
                                yield Completion(api, start_position=0 if len(words)==3 else -len(words[3]))
                        return
                # /config api edit ...
                if words[2] == "edit":
                    apis = list(config.get("online", {}).get("apis", {}).keys())
                    # Suggest API names
                    if len(words) == 3 or (len(words) == 4 and not text.endswith(" ")):
                        for api in apis:
                            if len(words) == 3 or api.startswith(words[3].lower()):
                                yield Completion(api, start_position=0 if len(words)==3 else -len(words[3]))
                        return
                    # Suggest key/model after API name
                    if len(words) == 4 or (len(words) == 5 and not text.endswith(" ")):
                        for opt in ["key", "model"]:
                            if len(words) == 4 or opt.startswith(words[4].lower()):
                                yield Completion(opt, start_position=0 if len(words)==4 else -len(words[4]))
                        return
                    # Suggest <value> placeholder
                    if len(words) == 5:
                        yield Completion("<value>", start_position=0)
                        return
            # /config prev_cmds ...
            if words[1] == "prev_cmds":
                if len(words) == 2 or (len(words) == 3 and not text.endswith(" ")):
                    for i in range(0, 11):
                        val = str(i)
                        if len(words) == 2 or val.startswith(words[2]):
                            yield Completion(val, start_position=0 if len(words)==2 else -len(words[2]))
                    return
        # /prompt theme autocomplete
        if words[0] == "/prompt":
            themes = ["default", "pwd", "mood"]
            if len(words) == 1 or (len(words) == 2 and not text.endswith(" ")):
                for theme in themes:
                    if len(words) == 1 or theme.startswith(words[1].lower()):
                        yield Completion(theme, start_position=0 if len(words)==1 else -len(words[1]))
                return

def get_custom_prompt():
    """Generate the prompt starting with AiSH based on the selected theme."""
    theme = config["aish"]["prompt_theme"]
    if theme == "default":
        return "AiSH> "
    elif theme == "pwd":
        current_dir = os.getcwd()
        return f"AiSH {current_dir}> "
    elif theme == "mood":
        if last_error is None:
            return "AiSH 😊> "
        else:
            return "AiSH 😞> "
    return "AiSH> "  # Fallback to default

def get_history_path():
    """Return the path for the history file, same location as config file, cross-platform."""
    config_path = get_config_path()
    config_dir = os.path.dirname(config_path)
    if os.name == 'nt':
        return os.path.join(config_dir, "aish_history.txt")
    else:
        return os.path.join(config_dir, ".aish_history")

def classify_intent(prompt, last_error):
    """Classify the user intent based on input and context."""
    prompt_lower = prompt.lower().strip()
    if last_error and ("retry" in prompt_lower or "fix" in prompt_lower or "try again" in prompt_lower):
        return "error_retry"
    elif prompt_lower.endswith("?") or any(word in prompt_lower for word in ["what", "where", "how", "who", "why"]):
        return "question"
    elif any(word in prompt_lower for word in ["create", "build", "write", "make", "setup"]) and len(prompt.split()) > 2:
        return "autonomous_task"
    else:
        return "single_command"

def handle_aish_command(command):
    """Handle AiSH-specific commands with shortcuts and help menus."""
    global verbose_mode, config
    command = command.lower()
    parts = command.split()

    if command in ("help", "h"):
        print(f"{Fore.CYAN}AiSH Commands:{Style.RESET_ALL}")
        print(f"{Fore.WHITE}/help or /h      - Show this help{Style.RESET_ALL}")
        print(f"{Fore.WHITE}/verbose or /v   - Toggle verbose mode (current: {verbose_mode}){Style.RESET_ALL}")
        print(f"{Fore.WHITE}/config or /c    - Configure settings (api, prev_cmds){Style.RESET_ALL}")
        print(f"{Fore.WHITE}/prompt [theme]  - Set prompt theme (default, pwd, mood){Style.RESET_ALL}")
        print(f"{Fore.WHITE}/exit or /e      - Exit AiSH{Style.RESET_ALL}")
        print(f"{Fore.WHITE}!cmd             - Execute raw shell command{Style.RESET_ALL}")
        print(f"{Fore.WHITE}Ctrl+C           - Exit or stop task{Style.RESET_ALL}")
    elif command in ("verbose", "v"):
        verbose_mode = not verbose_mode
        print(f"{Fore.YELLOW}Verbose mode: {verbose_mode}{Style.RESET_ALL}")
    elif parts[0] in ("config", "c"):
        # Show all config sections and options dynamically
        if len(parts) == 1:
            print(f"{Fore.CYAN}Usage: /config <section> <option> [value]{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Sections and options:{Style.RESET_ALL}")
            print(f"  api: current, fallback, edit{Style.RESET_ALL}")
            print(f"  prev_cmds: <int> (0-10){Style.RESET_ALL}")
            print(f"{Fore.WHITE}Examples:{Style.RESET_ALL}")
            print(f"  /config api current groq{Style.RESET_ALL}")
            print(f"  /config api fallback gemini{Style.RESET_ALL}")
            print(f"  /config api edit openrouter key sk-...{Style.RESET_ALL}")
            print(f"  /config api edit groq model llama-70b{Style.RESET_ALL}")
            print(f"  /config prev_cmds 3{Style.RESET_ALL}")
            return
        section = parts[1]
        if section == "api":
            apis = list(config["online"]["apis"].keys())
            if len(parts) == 2:
                current_api = config["online"].get("current", "None")
                fallback_api = config["online"].get("fallback", "None")
                print(f"{Fore.WHITE}Current API: {current_api}{Style.RESET_ALL}")
                print(f"{Fore.WHITE}Fallback API: {fallback_api}{Style.RESET_ALL}")
                print(f"{Fore.WHITE}Available APIs: {', '.join(apis)}{Style.RESET_ALL}\n")
                print(f"{Fore.WHITE}Next options:{Style.RESET_ALL}")
                print(f"  /config api current <api_name>")
                print(f"  /config api fallback <api_name>")
                print(f"  /config api edit <api_name> <key|model> <value>{Style.RESET_ALL}")
                print(f"{Fore.WHITE}Examples:{Style.RESET_ALL}")
                print(f"  /config api current groq{Style.RESET_ALL}")
                print(f"  /config api edit openrouter key sk-...{Style.RESET_ALL}")
                print(f"  /config api edit groq model llama-70b{Style.RESET_ALL}")
                return
            if len(parts) == 3:
                if parts[2] in ("current", "fallback"):
                    print(f"{Fore.RED}Missing API name for {parts[2]}.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api {parts[2]} <api_name>{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Available APIs: {', '.join(apis)}{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api {parts[2]} groq{Style.RESET_ALL}")
                    return
                elif parts[2] == "edit":
                    print(f"{Fore.RED}Missing API name and field for edit.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api edit <api_name> <key|model> <value>{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Available APIs: {', '.join(apis)}{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit openrouter key sk-...{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit groq model llama-70b{Style.RESET_ALL}")
                    return
                else:
                    print(f"{Fore.RED}Unknown subcommand: {parts[2]}{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Valid options: current, fallback, edit{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Examples:{Style.RESET_ALL}")
                    print(f"  /config api current groq{Style.RESET_ALL}")
                    print(f"  /config api fallback gemini{Style.RESET_ALL}")
                    print(f"  /config api edit openrouter key sk-...{Style.RESET_ALL}")
                    return
            if len(parts) == 4 and parts[2] in ("current", "fallback"):
                api_type = parts[2]
                api_name = parts[3]
                if api_name in apis:
                    config["online"][api_type] = api_name
                    try:
                        with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                            yaml.dump(config, f)
                        print(f"{Fore.YELLOW}{api_type.capitalize()} API set to: {api_name}{Style.RESET_ALL}")
                    except Exception as e:
                        print(f"{Fore.RED}Error saving configuration: {e}{Style.RESET_ALL}")
                else:
                    print(f"{Fore.RED}Invalid API name. Available APIs: {', '.join(apis)}{Style.RESET_ALL}")
                return
            if len(parts) >= 3 and parts[2] == "edit":
                if len(parts) == 3:
                    print(f"{Fore.RED}Usage: /config api edit <api_name> <key|model> <value>{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Available APIs: {', '.join(apis)}{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit openrouter key sk-...{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit groq model llama-70b{Style.RESET_ALL}")
                    return
                api_name = parts[3] if len(parts) > 3 else None
                if not api_name or api_name not in apis:
                    print(f"{Fore.RED}Invalid or missing API name.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Available APIs: {', '.join(apis)}{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api edit <api_name> <key|model> <value>{Style.RESET_ALL}")
                    return
                if len(parts) == 4:
                    print(f"{Fore.RED}Missing field. Use 'key' or 'model'.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api edit {api_name} <key|model> <value>{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit {api_name} key sk-...{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Example: /config api edit {api_name} model llama-70b{Style.RESET_ALL}")
                    return
                field = parts[4]
                if field not in ("key", "model"):
                    print(f"{Fore.RED}Invalid field: {field}.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Valid fields: key, model{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api edit {api_name} <key|model> <value>{Style.RESET_ALL}")
                    return
                if len(parts) < 6:
                    print(f"{Fore.RED}Missing value for {field}.{Style.RESET_ALL}")
                    print(f"{Fore.WHITE}Usage: /config api edit {api_name} {field} <value>{Style.RESET_ALL}")
                    return
                value = " ".join(parts[5:])
                if field == "key":
                    encrypted = encrypt_api_key(value, api_name)
                    config["online"]["apis"][api_name]["api_key"] = encrypted
                    with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                        yaml.dump(config, f)
                    print(f"{Fore.YELLOW}API key for {api_name} updated and encrypted.{Style.RESET_ALL}")
                elif field == "model":
                    config["online"]["apis"][api_name]["model"] = value
                    with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                        yaml.dump(config, f)
                    print(f"{Fore.YELLOW}Model for {api_name} updated.{Style.RESET_ALL}")
                return
            print(f"{Fore.RED}Usage: /config api <current|fallback|edit> ...{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Valid options: current, fallback, edit{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Examples:{Style.RESET_ALL}")
            print(f"  /config api current groq{Style.RESET_ALL}")
            print(f"  /config api fallback gemini{Style.RESET_ALL}")
            print(f"  /config api edit openrouter key sk-...{Style.RESET_ALL}")
            return
        elif section == "prev_cmds":
            if len(parts) == 2:
                current_val = config["aish"].get("prev_cmds_limit", 5)
                print(f"{Fore.WHITE}Current prev_cmds value: {current_val}{Style.RESET_ALL}")
                print(f"{Fore.WHITE}Usage: /config prev_cmds <int> (0-10){Style.RESET_ALL}")
                print(f"{Fore.WHITE}Example: /config prev_cmds 3{Style.RESET_ALL}")
                return
            if len(parts) == 3 and parts[2].isdigit():
                limit = int(parts[2])
                if 0 <= limit <= 10:
                    config["aish"]["prev_cmds_limit"] = limit
                    with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                        yaml.dump(config, f)
                    print(f"{Fore.YELLOW}Previous commands limit set to {limit}{Style.RESET_ALL}")
                else:
                    print(f"{Fore.RED}Invalid number. Use 0 to 10.{Style.RESET_ALL}")
                return
            print(f"{Fore.RED}Usage: /config prev_cmds <int> (0-10){Style.RESET_ALL}")
            print(f"{Fore.WHITE}Example: /config prev_cmds 3{Style.RESET_ALL}")
            return
        else:
            print(f"{Fore.RED}Unknown config section: {section}{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Sections: api, prev_cmds{Style.RESET_ALL}")
            return
    elif parts[0] == "prompt":
        if len(parts) == 1:
            print(f"{Fore.RED}Usage: /prompt [theme]{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Changes the prompt theme. Available themes: default, pwd, mood{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Examples:{Style.RESET_ALL}")
            print(f"{Fore.WHITE}  /prompt default{Style.RESET_ALL}")
            print(f"{Fore.WHITE}  /prompt pwd{Style.RESET_ALL}")
            print(f"{Fore.WHITE}  /prompt mood{Style.RESET_ALL}")
        elif len(parts) == 2:
            theme = parts[1]
            if theme in ("default", "pwd", "mood"):
                config["aish"]["prompt_theme"] = theme
                with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                    yaml.dump(config, f)
                print(f"{Fore.YELLOW}Prompt theme set to: {theme}{Style.RESET_ALL}")
            else:
                print(f"{Fore.RED}Invalid theme. Available themes: default, pwd, mood{Style.RESET_ALL}")
        else:
            print(f"{Fore.RED}Usage: /prompt [theme]{Style.RESET_ALL}")
    elif command in ("exit", "e", "quit"):
        print(f"{Fore.MAGENTA}🛑 Exiting AiSH...{Style.RESET_ALL}")
        sys.exit(0)
    else:
        print(f"{Fore.RED}Unknown command: {command}{Style.RESET_ALL}")

# Add error retry logic and AI analysis for errors
MAX_ERROR_RETRIES = config["aish"].get("error_retries", 3)  # Default to 3 retries if not set in .aishrc

def handle_error_with_ai(command, error, retries_left, attempts=None):
    """Handle errors by passing them to the AI for analysis and retry, with full attempt history."""
    if attempts is None:
        attempts = []
    if retries_left <= 0:
        print(f"{Fore.RED}❌ Task failed after maximum retries.{Style.RESET_ALL}")
        return False, error

    print(f"{Fore.YELLOW}⚠️ Retrying due to error: {error} (Retries left: {retries_left}){Style.RESET_ALL}")

    # Add the last attempt to the attempts list
    attempts = attempts + [{"command": command, "error": error}]

    # Use the correct AI provider for retries
    system_info = get_system_info()
    mode = config["aish"].get("mode", "online")
    ai_method = None
    if mode == "online":
        ai_method = config.get("online", {}).get("current", "groq")
    elif mode == "offline":
        ai_method = "ollama"
    else:
        # fallback to groq if unknown mode
        ai_method = "groq"

    # Create a prompt for AI to analyze the error and suggest a fix, with all attempts
    full_prompt = PromptFactory.error_retry_prompt(
        command,
        system_info,
        history,
        config["aish"].get("prev_cmds_limit", 5),
        error,
        attempts=attempts
    )
    ai_response = clean_response(query_ai(full_prompt, method=ai_method))

    # Extract the command from AI response
    cmd_match = re.search(r"CMD:\s*'([^']*)'", ai_response)
    if cmd_match:
        corrected_command = cmd_match.group(1)
    else:
        cmd_match = re.search(r"CMD:\s*(.+)", ai_response)
        corrected_command = cmd_match.group(1).strip() if cmd_match else None

    if corrected_command:
        print(f"{Fore.YELLOW}📢 AI Suggested Fix: {corrected_command}{Style.RESET_ALL}")
        output, new_error = execute_command(corrected_command)
        if new_error:
            return handle_error_with_ai(corrected_command, new_error, retries_left - 1, attempts)
        else:
            return True, output
    else:
        print(f"{Fore.RED}❌ AI could not suggest a fix: {ai_response}{Style.RESET_ALL}")
        return False, error

# Migration: encrypt unencrypted keys in .aishrc
for api_name, api_info in config.get("online", {}).get("apis", {}).items():
    key = api_info.get("api_key", "")
    if key and not (key.startswith("gAAAA") or key.startswith("b'")):
        encrypted = encrypt_api_key(key, api_name)
        config["online"]["apis"][api_name]["api_key"] = encrypted
        with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
            yaml.dump(config, f)

def main():
    """Run the AiSH interactive shell with enhanced input features."""
    global last_error, task_running, verbose_mode, history, config
    
    session = PromptSession(
        get_custom_prompt,
        style=prompt_style,
        history=FileHistory(get_history_path()),
        auto_suggest=AutoSuggestFromHistory(),
        completer=AiSHCompleter(),
        enable_history_search=True,
        multiline=False
    )
    
    # Placeholder system info (replace with actual system data from system_info.py)
    system_info = get_system_info()  # Assuming this is your function
    mem_mb = system_info['Total Memory (Bytes)'] // 1024**2
    mem_free_mb = system_info['Available Memory (Bytes)'] // 1024**2

    # Dynamic system details
    username = os.getlogin()
    current_dir = os.getcwd()
    current_time = datetime.now().strftime("%H:%M:%S")

    # Print the logo
    print(f"{Fore.CYAN}{Style.BRIGHT}")
    print("╔═╗┬╔═╗╦ ╦")
    print("╠═╣│╚═╗╠═╣")
    print("╩ ╩┴╚═╝╩ ╩")
    print(f"{Style.RESET_ALL}")

    # Print version line
    print(f"{Fore.CYAN}=== 🌟 {Style.BRIGHT}AiSH v0.1{Style.NORMAL} ==={Style.RESET_ALL}")

    # Precompute strings to avoid f-string nesting issues
    os_line = f"{system_info['OS']} {system_info['OS Version']}"
    cpu_line = f"{system_info['CPU Count']} cores @ {system_info['CPU Usage (%)']:.1f}%"
    ram_line = f"{mem_mb} MB, {mem_free_mb} free"
    user_line = username
    dir_line = current_dir
    time_line = current_time

    # Finding the max size of the lines
    max_size = max(len(os_line), len(cpu_line), len(ram_line), len(user_line), len(dir_line), len(time_line))
    
    # Print system info with borders
    print(f"{Fore.CYAN}╔{'═' * (max_size + 16)}╗{Style.RESET_ALL}")
    print(f"{Fore.CYAN}║ {Fore.CYAN}💻 OS       : {Fore.WHITE}{os_line}{Style.RESET_ALL}{' ' * (max_size - len(os_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}║ {Fore.CYAN}⚙️  CPU      : {Fore.WHITE}{cpu_line}{Style.RESET_ALL}{' ' * (max_size - len(cpu_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}║ {Fore.CYAN}📦 RAM      : {Fore.WHITE}{ram_line}{Style.RESET_ALL}{' ' * (max_size - len(ram_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}║ {Fore.CYAN}🖥️  User     : {Fore.WHITE}{user_line}{Style.RESET_ALL}{' ' * (max_size - len(user_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}║ {Fore.CYAN}📂 Dir      : {Fore.WHITE}{dir_line}{Style.RESET_ALL}{' ' * (max_size - len(dir_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}║ {Fore.CYAN}⏰ Time     : {Fore.WHITE}{time_line}{Style.RESET_ALL}{' ' * (max_size - len(time_line))} {Fore.CYAN}║")
    print(f"{Fore.CYAN}╚{'═' * (max_size + 16)}╝{Style.RESET_ALL}")

    # Print help message
    print(f"{Fore.WHITE}Type '/help' for commands. Use ↑↓ for history, ←→ to edit, Ctrl+C to exit.{Style.RESET_ALL}")

    
    while True:
        try:
            prompt = session.prompt()
        except KeyboardInterrupt:
            print(f"{Fore.MAGENTA}🛑 Exiting AiSH...{Style.RESET_ALL}")
            sys.exit(0)

        if not prompt.strip():
            continue

        if prompt.startswith("/"):
            handle_aish_command(prompt[1:].strip())
        elif prompt.startswith("!"):
            raw_command = prompt[1:].strip()
            cmd_parts = raw_command.split()
            if cmd_parts and cmd_parts[0] == "cd":
                if len(cmd_parts) == 1:
                    try:
                        os.chdir(os.path.expanduser("~"))
                        last_error = None
                    except Exception as e:
                        last_error = str(e)
                        print(f"{Fore.RED}❌ Error: {last_error}{Style.RESET_ALL}")
                elif len(cmd_parts) == 2:
                    directory = cmd_parts[1]
                    try:
                        os.chdir(directory)
                        last_error = None
                    except Exception as e:
                        last_error = str(e)
                        print(f"{Fore.RED}❌ Error: {last_error}{Style.RESET_ALL}")
                else:
                    last_error = "cd: too many arguments"
                    print(f"{Fore.RED}❌ {last_error}{Style.RESET_ALL}")
            else:
                if verbose_mode:
                    print(f"{Fore.YELLOW}📢 Executing: {raw_command}{Style.RESET_ALL}")
                output, error = execute_command(raw_command)
                if error:
                    success, result = handle_error_with_ai(raw_command, error, MAX_ERROR_RETRIES)
                    if not success:
                        print(f"{Fore.RED}❌ Error: {result}{Style.RESET_ALL}")
                        last_error = result
                    else:
                        print(f"{Fore.WHITE}{result}{Style.RESET_ALL}")
                        last_error = None
                else:
                    print(f"{Fore.WHITE}{output}{Style.RESET_ALL}")
                    last_error = None
        else:
            system_info = get_system_info()
            intent = classify_intent(prompt, last_error)
            mode = config["aish"]["mode"]
            if mode == "online":
                current_api = config["online"]["current"]
                ai_method = current_api
            else:
                ai_method = "ollama"
            
            if intent == "single_command":
                full_prompt = PromptFactory.single_command_prompt(prompt, system_info, history, config["aish"]["prev_cmds_limit"])
                ai_response = clean_response(query_ai(full_prompt, method=ai_method))
                ai_response = ai_response.splitlines()[0].strip()
                cmd_match = re.search(r"CMD:\s*'([^']*)'", ai_response)
                if cmd_match:
                    command = cmd_match.group(1)
                else:
                    cmd_match = re.search(r"CMD:\s*(.+)", ai_response)
                    if cmd_match:
                        command = cmd_match.group(1).strip()
                    else:
                        command = None

                if command:
                    if verbose_mode:
                        print(f"{Fore.YELLOW}📢 Executing: {command}{Style.RESET_ALL}")
                    output, error = execute_command(command)
                    if error:
                        # Retry using AI error correction logic
                        success, result = handle_error_with_ai(command, error, MAX_ERROR_RETRIES)
                        if not success:
                            print(f"{Fore.RED}❌ Error: {result}{Style.RESET_ALL}")
                            last_error = result
                            execution_result = f"Error: {result}"
                        else:
                            print(f"{Fore.WHITE}{result}{Style.RESET_ALL}")
                            last_error = None
                            execution_result = result
                    else:
                        print(f"{Fore.WHITE}{output}{Style.RESET_ALL}")
                        last_error = None
                        execution_result = output
                    history.append({
                        'user_input': prompt,
                        'intent': 'single_command',
                        'ai_response': ai_response,
                        'execution_result': execution_result
                    })
                else:
                    print(f"{Fore.BLUE}ℹ️ {ai_response}{Style.RESET_ALL}")
                    history.append({
                        'user_input': prompt,
                        'intent': 'single_command',
                        'ai_response': ai_response,
                        'execution_result': None
                    })
            elif intent == "question":
                full_prompt = PromptFactory.question_prompt(prompt, system_info, history, config["aish"]["prev_cmds_limit"])
                ai_response = clean_response(query_ai(full_prompt, method=ai_method))
                ai_response = ai_response.splitlines()[0].strip()
                print(f"{Fore.BLUE}ℹ️ {ai_response}{Style.RESET_ALL}")
                history.append({
                    'user_input': prompt,
                    'intent': 'question',
                    'ai_response': ai_response,
                    'execution_result': None
                })
            elif intent == "error_retry":
                full_prompt = PromptFactory.error_retry_prompt(prompt, system_info, history, config["aish"]["prev_cmds_limit"], last_error)
                ai_response = clean_response(query_ai(full_prompt, method=ai_method))
                ai_response = ai_response.splitlines()[0].strip()
                cmd_match = re.search(r"CMD:\s*'([^']*)'", ai_response)
                if cmd_match:
                    command = cmd_match.group(1)
                else:
                    cmd_match = re.search(r"CMD:\s*(.+)", ai_response)
                    if cmd_match:
                        command = cmd_match.group(1).strip()
                    else:
                        command = None

                if command:
                    if verbose_mode:
                        print(f"{Fore.YELLOW}📢 Executing: {command}{Style.RESET_ALL}")
                    output, error = execute_command(command)
                    if error:
                        print(f"{Fore.RED}❌ Error: {error}{Style.RESET_ALL}")
                        last_error = error
                        execution_result = f"Error: {error}"
                    else:
                        print(f"{Fore.WHITE}{output}{Style.RESET_ALL}")
                        last_error = None
                        execution_result = output
                    history.append({
                        'user_input': prompt,
                        'intent': 'error_retry',
                        'ai_response': ai_response,
                        'execution_result': execution_result
                    })
                else:
                    print(f"{Fore.BLUE}ℹ️ {ai_response}{Style.RESET_ALL}")
                    history.append({
                        'user_input': prompt,
                        'intent': 'error_retry',
                        'ai_response': ai_response,
                        'execution_result': None
                    })
            else:  # autonomous_task
                task_running = True
                success, message = process_autonomous_task(prompt, system_info, verbose=verbose_mode)
                history.append({
                    'user_input': prompt,
                    'intent': 'autonomous_task',
                    'ai_response': message,
                    'execution_result': None
                })
                task_running = False
                continue

        # Limit history to last 10 interactions
        history = history[-10:]

if __name__ == "__main__":
    main()