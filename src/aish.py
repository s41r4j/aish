from colorama import Fore, Style, init
from system_info import get_system_info
from utils import execute_command
from task_manager import process_autonomous_task
from ai_interface import query_ai
from prompt_factory import PromptFactory, format_history, clean_response
from config import config
import re
import sys
import signal
from prompt_toolkit import PromptSession
from prompt_toolkit.history import FileHistory
from prompt_toolkit.auto_suggest import AutoSuggestFromHistory
from prompt_toolkit.styles import Style as PromptStyle
import os
from datetime import datetime

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
        print(f"{Fore.WHITE}/prevcmds <int>  - Set number of previous commands in prompts (0-10){Style.RESET_ALL}")
        print(f"{Fore.WHITE}/prompt [theme]  - Set prompt theme (default, pwd, mood){Style.RESET_ALL}")
        print(f"{Fore.WHITE}/exit or /quit   - Exit AiSH{Style.RESET_ALL}")
        print(f"{Fore.WHITE}!cmd             - Execute raw shell command{Style.RESET_ALL}")
        print(f"{Fore.WHITE}Ctrl+C           - Exit or stop task{Style.RESET_ALL}")
    elif command in ("verbose", "v"):
        verbose_mode = not verbose_mode
        print(f"{Fore.YELLOW}Verbose mode: {verbose_mode}{Style.RESET_ALL}")
    elif parts[0] == "prevcmds":
        if len(parts) == 1 or not parts[1].isdigit():
            print(f"{Fore.RED}Usage: /prevcmds <int>{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Sets the number of previous commands to include (0-10).{Style.RESET_ALL}")
            print(f"{Fore.WHITE}Example: /prevcmds 3{Style.RESET_ALL}")
        else:
            limit = int(parts[1])
            if 0 <= limit <= 10:
                config["aish"]["prev_cmds_limit"] = limit
                with open(os.path.join(os.path.expanduser("~"), ".aishrc"), "w") as f:
                    yaml.dump(config, f)
                print(f"{Fore.YELLOW}Previous commands limit set to {limit}{Style.RESET_ALL}")
            else:
                print(f"{Fore.RED}Invalid number. Use 0 to 10.{Style.RESET_ALL}")
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
    elif command in ("exit", "quit"):
        print(f"{Fore.MAGENTA}🛑 Exiting AiSH...{Style.RESET_ALL}")
        sys.exit(0)
    else:
        print(f"{Fore.RED}Unknown command: {command}{Style.RESET_ALL}")

def main():
    """Run the AiSH interactive shell with enhanced input features."""
    global last_error, task_running, verbose_mode, history, config
    
    session = PromptSession(
        get_custom_prompt,
        style=prompt_style,
        history=FileHistory(".aish_history"),
        auto_suggest=AutoSuggestFromHistory(),
        enable_history_search=True,
        multiline=False
    )
    
    # system_info = get_system_info()
    # mem_mb = system_info['Total Memory (Bytes)'] // 1024**2
    # mem_free_mb = system_info['Available Memory (Bytes)'] // 1024**2
    # print(f"{Fore.CYAN}=== 🌟 AiSH v0.1 ==={Style.RESET_ALL}")
    # print(f"{Fore.WHITE}  💻 OS       : {system_info['OS']} {system_info['OS Version'][:20]}{Style.RESET_ALL}")
    # print(f"{Fore.WHITE}  ⚙️  CPU      : {system_info['CPU Count']} cores @ {system_info['CPU Usage (%)']:.1f}%{Style.RESET_ALL}")
    # print(f"{Fore.WHITE}  📦 RAM      : {mem_mb} MB total, {mem_free_mb} MB free{Style.RESET_ALL}")
    # print(f"{Fore.WHITE}Type '/help' for commands. Use ↑↓ for history, ←→ to edit, Ctrl+C to exit/stop.{Style.RESET_ALL}")


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
                    print(f"{Fore.RED}❌ Error: {error}{Style.RESET_ALL}")
                    last_error = error
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
                        print(f"{Fore.RED}❌ Error: {error}{Style.RESET_ALL}")
                        last_error = error
                        execution_result = f"Error: {error}"
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