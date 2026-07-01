# AiSH Bundling

AiSH packages the app binary and `llama-cli` together. The GGUF model is not
embedded in the binary; it is downloaded into the user data directory on first
run after confirmation.

## First-run flow

1. User runs `aish` from any directory.
2. AiSH creates its home directory if needed:
   - Linux/macOS: `~/.aish`
   - Windows: `%APPDATA%\AiSH`
3. AiSH loads or creates `config.toml`.
4. AiSH checks for the configured model:
   - default path: `~/.aish/models/aish.gguf`
   - default source:
     `https://huggingface.co/Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF/resolve/main/qwen2.5-coder-1.5b-instruct-q2_k.gguf`
5. If the model is missing, AiSH prompts before downloading.
6. If the user declines, AiSH exits without running a request.
7. If the user accepts, AiSH downloads the model and then starts normally.
8. AiSH resolves `llama-cli` from the bundle and uses it for local inference.

`--yes` skips confirmation for the model download and for non-blocked commands.

## Runtime lookup

AiSH resolves `llama-cli` in this order:

1. `AISH_LLAMA_BIN`, if set.
2. Runtime bundled next to the executable:
   - `runtime/llama-cli`
   - `../runtime/llama-cli`
   - Windows uses `llama-cli.exe`.
3. Installed runtime:
   - Linux/macOS: `~/.local/share/aish/runtime/llama-cli`
   - Windows: `%LOCALAPPDATA%\AiSH\runtime\llama-cli.exe`
4. `runtime_command` from config, defaulting to `llama-cli`.

This means a packaged install does not require the user to install llama.cpp.
The system `llama-cli` is only a fallback.

## Linux bundle

Build:

```sh
scripts/package-linux.sh
```

Output:

```text
dist/aish-linux-x86_64.tar.gz
```

Bundle layout:

```text
aish-linux-x86_64/
  bin/
    aish
    aish-download-model
  runtime/
    llama-cli
    other llama.cpp runtime files
  install.sh
```

Install:

```sh
tar -xzf dist/aish-linux-x86_64.tar.gz -C dist
dist/aish-linux-x86_64/install.sh
```

## Windows bundle

Build from Docker on Linux/macOS:

```sh
scripts/package-windows-docker.sh
```

Or build from a native Windows machine with Rust, Git, CMake, and MSVC Build
Tools:


```powershell
powershell -ExecutionPolicy Bypass -File scripts\package-windows.ps1
```

Output:

```text
dist\aish-windows-x86_64.zip
```

Bundle layout:

```text
aish-windows-x86_64\
  bin\
    aish.exe
  runtime\
    llama-cli.exe
    other llama.cpp runtime files
  install.ps1
```

Install:

```powershell
Expand-Archive dist\aish-windows-x86_64.zip -DestinationPath dist
powershell -ExecutionPolicy Bypass -File dist\aish-windows-x86_64\install.ps1
```

The installer creates an `aish.cmd` shim under `%USERPROFILE%\.local\bin` and
adds that directory to the user PATH if needed.
