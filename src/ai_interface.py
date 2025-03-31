import os
import requests
from groq import Groq
import google.generativeai as genai  # For Gemini API
from config import config

# Configuration
OLLAMA_API_URL = "http://localhost:11434/api/generate"
GROQ_API_KEY = config["online"]["apis"]["groq"]["api_key"]
GROQ_MODEL = config["online"]["apis"]["groq"]["model"]
GEMINI_API_KEY = config["online"]["apis"]["gemini"]["api_key"]
GEMINI_MODEL = config["online"]["apis"]["gemini"]["model"]
OLLAMA_MODEL = config["offline"]["ollama_model"]

# Initialize clients
client_groq = Groq(api_key=GROQ_API_KEY)
genai.configure(api_key=GEMINI_API_KEY)

def query_groq(prompt):
    """Query the groq API."""
    try:
        response = client_groq.chat.completions.create(
            model=GROQ_MODEL,
            messages=[{"role": "user", "content": prompt}],
            temperature=0.5,
            max_tokens=1024
        )
        return response.choices[0].message.content.strip()
    except Exception as e:
        return f"Error: {str(e)}"

def query_gemini(prompt):
    """Query the Gemini API."""
    try:
        model = genai.GenerativeModel(GEMINI_MODEL)
        response = model.generate_content(prompt)
        return response.text.strip()
    except Exception as e:
        return f"Error: {str(e)}"

def query_ollama_api(prompt):
    """Query Ollama via its REST API."""
    try:
        response = requests.post(OLLAMA_API_URL, json={"prompt": prompt, "model": OLLAMA_MODEL})
        response.raise_for_status()
        return response.json().get("command", "")
    except requests.RequestException as e:
        return f"Error: Failed to connect to Ollama API - {e}"

def query_ai(prompt, method="groq"):
    """Unified AI query function with method selection."""
    if method == "groq":
        return query_groq(prompt)
    elif method == "gemini":
        return query_gemini(prompt)
    elif method == "ollama":
        return query_ollama_api(prompt)
    else:
        return "Error: Invalid AI method specified"