import os
import yaml
import sys
import core.config as config

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

def merge_configs(default, user):
    """Recursively merge user config into default config."""
    if isinstance(default, dict) and isinstance(user, dict):
        for key, value in default.items():
            if key not in user:
                user[key] = value
            else:
                user[key] = merge_configs(value, user[key])
    return user

def load_config():
    """
    Load configuration from the config file.
    If the file doesn't exist, create it with defaults.
    If it exists, load it and merge with defaults to add any missing keys.
    Returns the loaded and potentially updated config dictionary.
    """
    default_config = {
        "aish": {
            "prev_cmds_limit": 5,
            "prompt_theme": "default",
            "mode": "online",
            "error_retries": 3
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
                },
                "openrouter": {
                    "api_key": "",
                    "model": ""
                }
            }
        },
        "offline": {
            "ollama_model": ""
        }
    }

    config_to_load = default_config.copy() # Start with defaults

    if os.path.exists(CONFIG_PATH):
        try:
            with open(CONFIG_PATH, "r") as f:
                user_config = yaml.safe_load(f)
                if user_config: # If file is not empty or malformed
                    config_to_load = merge_configs(default_config, user_config)
        except Exception as e:
            print(f"Warning: Could not read or parse config file at {CONFIG_PATH}: {e}")
            print("Using default configuration and attempting to rewrite the config file.")
            # config_to_load remains default_config
    
    # Always write back the potentially merged/updated config
    # This creates the file if it doesn't exist, or updates it if new keys were added
    try:
        with open(CONFIG_PATH, "w") as f:
            yaml.dump(config_to_load, f, default_flow_style=False)
        if not os.path.exists(CONFIG_PATH): # Should not happen if write was successful
             print(f"Created default config file at {CONFIG_PATH}. Please edit it with your API keys.")
        elif user_config is None or user_config != config_to_load: # If it was newly created or updated
             print(f"Config file at {CONFIG_PATH} has been initialized/updated.")
             if 'openrouter' not in (user_config.get('online', {}).get('apis', {}) if user_config else {}):
                 print("Please add your OpenRouter API key and preferred model to it.")

    except Exception as e:
        print(f"Error: Could not write config file at {CONFIG_PATH}: {e}")
        print("Using in-memory configuration for this session.")
        return config_to_load # Return whatever we have, even if saving failed

    return config_to_load

# Load config when the module is imported
config = load_config()