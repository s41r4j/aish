import subprocess

def execute_command(command):
    """Execute a shell command and return its output or error."""
    try:
        result = subprocess.run(
            command,
            shell=True,
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout, None
    except subprocess.CalledProcessError as e:
        return None, e.stderr