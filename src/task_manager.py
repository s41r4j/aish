from ai_interface import query_ai
from utils import execute_command
from colorama import Fore, Style

# Constants
MAX_RETRIES = 3

def create_task_prompt(user_input, system_info, last_error, task, steps):
    """Create a dynamic, feedback-oriented prompt for autonomous task processing."""
    prompt = (
        "You are AiSH, an AI-driven shell assistant running on a real machine. Your goal is to:\n"
        "- Autonomously process tasks by breaking them into executable shell commands.\n"
        "- Return exactly one line: either 'CMD: <command>' for the next step or 'Task completed' if done.\n"
        "- Do not include explanations, multi-line responses, or invalid commands.\n"
        f"System Info: {system_info}\n"
        "Instructions:\n"
        "- Use real shell commands (e.g., 'echo', 'ls', 'dir') based on the OS.\n"
        "- Avoid simulation; every 'CMD:' must be executable.\n"
        "- If a file or resource is missing, create it first.\n"
        "- Progress the task step-by-step, one 'CMD:' at a time.\n"
    )
    if steps:
        steps_history = "Completed Steps:\n"
        for i, step in enumerate(steps, 1):
            steps_history += f"{i}. CMD: {step['command']}\n"
            if 'output' in step:
                steps_history += f"Output: {step['output'][:100]}\n"
            elif 'error' in step:
                steps_history += f"Error: {step['error']}\n"
        prompt += steps_history
    if last_error:
        prompt += (
            f"\nError Feedback:\n"
            f"Last command failed: '{last_error}'. "
            "Analyze the error and return the next 'CMD:' to fix it.\n"
        )
    if task:
        prompt += (
            f"\nCurrent Task: {task}\n"
            "Return exactly one line: 'CMD: <command>' or 'Task completed'."
        )
    else:
        prompt += "\nStart the task with the first 'CMD:'.\n"
    prompt += f"\nUser Input: {user_input}"
    return prompt

def clean_response(response):
    """Remove <think> tags and their contents."""
    import re
    return re.sub(r'<think>.*?</think>', '', response, flags=re.DOTALL).strip()

def process_autonomous_task(user_input, system_info, verbose=False):
    """Autonomously process a multi-step task on the real machine with feedback."""
    task = user_input
    steps = []  # List of dicts with command and result
    error_retries = 0
    last_error = None

    print(f"{Fore.YELLOW}🚀 Starting task: {task}{Style.RESET_ALL}")
    while True:
        full_prompt = create_task_prompt(user_input if not steps else "", system_info, last_error, task, steps)
        ai_response = clean_response(query_ai(full_prompt, method="groq"))
        ai_response = ai_response.splitlines()[0].strip()  # Process only the first line

        if ai_response.startswith("Error:"):
            print(f"{Fore.RED}❌ {ai_response}{Style.RESET_ALL}")
            return False, ai_response
        elif ai_response.startswith("CMD:"):
            command = ai_response[4:].strip()
            if verbose:
                print(f"{Fore.YELLOW}📢 Executing: {command}{Style.RESET_ALL}")
            output, error = execute_command(command)
            if error:
                print(f"{Fore.RED}❌ Error: {error}{Style.RESET_ALL}")
                last_error = error
                error_retries += 1
                steps.append({'command': command, 'error': error})
                if error_retries >= MAX_RETRIES:
                    print(f"{Fore.RED}❌ Task failed after {MAX_RETRIES} retries.{Style.RESET_ALL}")
                    return False, f"Task failed after {MAX_RETRIES} retries."
            else:
                print(f"{Fore.WHITE}{output}{Style.RESET_ALL}")
                last_error = None
                error_retries = 0
                steps.append({'command': command, 'output': output})
        elif ai_response == "Task completed":
            print(f"{Fore.BLUE}✅ Task completed{Style.RESET_ALL}")
            return True, "Task completed"
        else:
            print(f"{Fore.RED}❌ Unexpected response: {ai_response}{Style.RESET_ALL}")
            return False, f"Unexpected response: {ai_response}"