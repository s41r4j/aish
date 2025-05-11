import re

def format_history(history, n):
    """Format the last n interactions for inclusion in prompts."""
    if not history:
        return "No recent interactions."
    recent = history[-n:]
    lines = []
    for entry in recent:
        lines.append(f"User: {entry['user_input']}")
        lines.append(f"AiSH: {entry['ai_response']}")
        if entry['execution_result']:
            lines.append(f"Result: {entry['execution_result'][:100]}")
    return "\n".join(lines)

class PromptFactory:
    """Dynamically generate task-specific prompts with context."""
    @staticmethod
    def base_prompt(system_info, history, prev_cmds_limit):
        formatted_history = format_history(history, prev_cmds_limit)
        return (
            "You are AiSH, an AI-driven shell assistant running on a real machine. "
            f"System Info: {system_info}\n"
            "Recent interactions:\n"
            f"{formatted_history}\n"
            "Instructions:\n"
            "- Use system info and recent interactions to adapt commands and responses.\n"
            "- Return exactly one line: either 'CMD: <command>' for executable commands or plain text for responses.\n"
            "- Do not include '<think>' tags, explanations, or multi-line responses.\n"
        )

    @staticmethod
    def single_command_prompt(user_input, system_info, history, prev_cmds_limit):
        return (
            PromptFactory.base_prompt(system_info, history, prev_cmds_limit) +
            "Goal: Interpret the user input as a single shell command.\n"
            "- Return exactly one line in the format: CMD: '<command>'\n"
            "- Enclose the command in single quotes.\n"
            "- Do not include any other text, explanations, or multiple lines.\n"
            "- Use 'ls' or 'dir' based on the OS, and target the home directory (~) when specified.\n"
            f"User Input: {user_input}\n"
        )

    @staticmethod
    def question_prompt(user_input, system_info, history, prev_cmds_limit):
        return (
            PromptFactory.base_prompt(system_info, history, prev_cmds_limit) +
            "Goal: Answer a question without executing a command.\n"
            f"User Input: {user_input}\n"
            "Provide a concise text response."
        )

    @staticmethod
    def error_retry_prompt(user_input, system_info, history, prev_cmds_limit, error, attempts=None):
        return (
            PromptFactory.base_prompt(system_info, history, prev_cmds_limit) +
            "Goal: Retry a failed command.\n"
            f"Previous Error: '{error}'\n"
            f"Original Input: {user_input}\n"
            "Analyze the error and return a corrected command in the format: CMD: '<corrected_command>'\n"
            "- Enclose the command in single quotes.\n"
            "- If you cannot correct the command, provide an explanation without the CMD format.\n"
        )

def clean_response(response):
    """Remove <think> tags and their contents."""
    return re.sub(r'<think>.*?</think>', '', response, flags=re.DOTALL).strip()