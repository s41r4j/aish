import os
import requests
from core.config import config
from core.utils import decrypt_api_key
import google.generativeai as genai  # For Gemini API
from openai import OpenAI
try:
    from groq import Groq
except ImportError:
    Groq = None

OLLAMA_API_URL = "http://localhost:11434/api/generate"

# Helper to get API key and model for a provider
def get_api_info(provider):
    if provider == "ollama":
        return None, config["offline"].get("ollama_model", "")
    online_apis = config.get("online", {}).get("apis", {})
    if provider in online_apis:
        api_key = decrypt_api_key(online_apis[provider].get("api_key", ""), provider) if online_apis[provider].get("api_key") else ""
        model = online_apis[provider].get("model", "")
        return api_key, model
    return None, None

# Dynamic client initialization
def get_client(provider):
    api_key, _ = get_api_info(provider)
    if provider == "groq" and Groq:
        return Groq(api_key=api_key)
    elif provider == "gemini":
        genai.configure(api_key=api_key)
        return None  # Gemini uses global config
    elif provider == "openrouter":
        return OpenAI(base_url="https://openrouter.ai/api/v1", api_key=api_key)
    return None

def query_groq(prompt):
    client = get_client("groq")
    _, model = get_api_info("groq")
    try:
        response = client.chat.completions.create(
            model=model,
            messages=[{"role": "user", "content": prompt}],
            temperature=0.5,
            max_tokens=1024
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        return f"Error: {str(e)}"

def query_gemini(prompt):
    _, model = get_api_info("gemini")
    try:
        model_obj = genai.GenerativeModel(model)
        response = model_obj.generate_content(prompt)
        return response.text.strip()
    except Exception as e:
        return f"Error: {str(e)}"

def query_ollama_api(prompt):
    _, model = get_api_info("ollama")
    try:
        response = requests.post(OLLAMA_API_URL, json={"prompt": prompt, "model": model})
        response.raise_for_status()
        return response.json().get("command", "")
    except requests.RequestException as e:
        return f"Error: Failed to connect to Ollama API - {e}"

def query_openrouter(prompt):
    client = get_client("openrouter")
    _, model = get_api_info("openrouter")
    try:
        response = client.chat.completions.create(
            extra_headers={
                "HTTP-Referer": "<YOUR_SITE_URL>",
                "X-Title": "AiSH"
            },
            extra_body={},
            model=model,
            messages=[{"role": "user", "content": prompt}]
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        return f"Error: {str(e)}"

def query_ai(prompt, method=None):
    """Unified AI query function with dynamic provider selection and fallback."""
    online_cfg = config.get("online", {})
    current = method or online_cfg.get("current")
    fallback = online_cfg.get("fallback")
    if current == "groq":
        result = query_groq(prompt)
    elif current == "gemini":
        result = query_gemini(prompt)
    elif current == "ollama":
        result = query_ollama_api(prompt)
    elif current == "openrouter":
        result = query_openrouter(prompt)
    else:
        return "Error: Invalid AI method specified"
    # Fallback if error
    if result.startswith("Error") and fallback and fallback != current:
        if fallback == "groq":
            return query_groq(prompt)
        elif fallback == "gemini":
            return query_gemini(prompt)
        elif fallback == "ollama":
            return query_ollama_api(prompt)
        elif fallback == "openrouter":
            return query_openrouter(prompt)
    return result