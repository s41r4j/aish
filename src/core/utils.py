import subprocess
from cryptography.fernet import Fernet
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.backends import default_backend
import base64

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

def get_fernet_key(api_name):
    password = api_name.encode()
    salt = api_name[::-1].encode()
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=100_000,
        backend=default_backend()
    )
    key = base64.urlsafe_b64encode(kdf.derive(password))
    return key

def encrypt_api_key(api_key, api_name):
    f = Fernet(get_fernet_key(api_name))
    return f.encrypt(api_key.encode()).decode()

def decrypt_api_key(encrypted, api_name):
    f = Fernet(get_fernet_key(api_name))
    return f.decrypt(encrypted.encode()).decode()