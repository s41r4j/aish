import core.system_info as system_info
import core.utils as utils
import platform
import psutil

def get_system_info():
    """Retrieve system information for AI context."""
    return {
        "OS": platform.system(),
        "OS Version": platform.version(),
        "CPU Count": psutil.cpu_count(),
        "CPU Usage (%)": psutil.cpu_percent(interval=1),
        "Total Memory (Bytes)": psutil.virtual_memory().total,
        "Available Memory (Bytes)": psutil.virtual_memory().available,
    }