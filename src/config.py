import os
import yaml
import sys

def get_config_path():
    # Determine the appropriate config file path based on the operating system.
    # - Windows: Uses %APPDATA%\aishrc.yaml (e.g., C:\Users\<username>\AppData\Roaming\aishrc.yaml)
    # - Linux/macOS: Uses ~/.aishrc (e.g., /home/<username>/.aishrc)
    
    if os.name == 'nt':  # Windows
        config_dir = os.getenv('APPDATA', os.path.expanduser("~"))
        return os.path.join(config_dir, "aishrc.yaml")
    else:  # Linux, macOS, etc.
        return os.path.join(os.path.expanduser("~"), ".aishrc")

CONFIG_PATH = get_config_path()

def load_config():
    """
    Load configuration from the config file or create it with defaults if not found.
    Returns the loaded config dictionary.
    """
    # Define default configuration with proper YAML-compatible structure
    default_config = {
        "aish": {
            "prev_cmds_limit": 5,
            "prompt_theme": "default",
            "mode": "online"
        },
        "online": {
            "current": "groq",
            "fallback": "gemini",
            "apis": {
                "groq": {
                    "api_key": "",
                    "model": ""
                },
                "gemini": {
                    "api_key": "",
                    "model": ""
                }
            }
        },
        "offline": {
            "ollama_model": ""
        }
    }

    # Check if the config file exists
    if not os.path.exists(CONFIG_PATH):
        try:
            # Create the file with default config
            with open(CONFIG_PATH, "w") as f:
                yaml.dump(default_config, f, default_flow_style=False)
            print(f"Created default config file at {CONFIG_PATH}. Please edit it with your API keys.")
        except Exception as e:
            # Handle errors (e.g., permission denied) and fall back to default config
            print(f"Warning: Could not create config file at {CONFIG_PATH}: {e}")
            print("Using default configuration.")
            return default_config

    # Load the config file
    try:
        with open(CONFIG_PATH, "r") as f:
            return yaml.safe_load(f)
    except Exception as e:
        # Handle errors (e.g., file corrupted) and fall back to default config
        print(f"Error: Could not read config file at {CONFIG_PATH}: {e}")
        print("Using default configuration.")
        return default_config

# Load config when the module is imported
config = load_config()