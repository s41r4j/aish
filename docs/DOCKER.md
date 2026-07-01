# AiSH Docker Setup

AiSH runs entirely in Docker. The Rust binary and `llama-cli` are installed in
the image, while the GGUF model, configuration, history, logs, and cache are
stored in the persistent `aish-home` Docker volume.

## Model

- Repository: `Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF`
- File: `qwen2.5-coder-1.5b-instruct-q2_k.gguf`
- Container path: `/home/aish/.aish/models/aish.gguf`
- Config path: `/home/aish/.aish/config.toml`

The repository is public, so `HF_TOKEN` is not required for the current model.
Passing `-e HF_TOKEN="$HF_TOKEN"` is still supported and is useful if the
configured model is changed to a gated or private repository.

## Build

Start Docker Desktop, then run from the repository root:

```bash
docker build -t aish-beta-llama .
docker volume create aish-home
```

Rebuilding the image does not remove or replace the `aish-home` volume.

## Download the Model

Download directly into the persistent Docker volume:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  --entrypoint aish-download-model \
  aish-beta-llama
```

AiSH also downloads the configured model automatically on first use when the
model path does not exist.

## Open the Docker Shell

```bash
docker run --rm -it \
  -v aish-home:/home/aish/.aish \
  --entrypoint /bin/bash \
  aish-beta-llama
```

Inside the container:

```bash
aish
```

Use preview-only mode while validating generated commands:

```bash
aish --no-exec "show the 20 largest files recursively"
```

Use natural-language summaries instead of direct command output:

```bash
aish --natural-output
```

## Run AiSH Directly

Interactive:

```bash
docker run --rm -it \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest
```

Interactive with natural-language output enabled:

```bash
docker run --rm -it \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest \
  --natural-output
```

Preview real-model command generation without executing it:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest \
  --no-exec "find all Python files recursively"
```

Test harmless execution with raw output:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest \
  "show current directory"
```

Test harmless execution with summarized output:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest \
  --natural-output "show current directory"
```

The previous token-based form remains valid:

```bash
docker run --rm -it \
  -e HF_TOKEN="$HF_TOKEN" \
  -v aish-home:/home/aish/.aish \
  aish-beta-llama:latest
```

## Configuration

Inspect the active configuration:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  --entrypoint /bin/bash \
  aish-beta-llama \
  -lc 'cat /home/aish/.aish/config.toml'
```

Read it from an interactive container shell:

```bash
sed -n '1,200p' ~/.aish/config.toml
```

The default model settings are:

```toml
model_download_dir = "/home/aish/.aish/models"
model_path = "/home/aish/.aish/models/aish.gguf"
model_repo = "Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF"
model_file = "qwen2.5-coder-1.5b-instruct-q2_k.gguf"
```

Existing volumes retain their old `config.toml`. To update one without deleting
its history or model, open the Docker shell and edit only these four values.

## Volume Maintenance

Show the downloaded model:

```bash
docker run --rm \
  -v aish-home:/home/aish/.aish \
  --entrypoint /bin/bash \
  aish-beta-llama \
  -lc 'ls -lh /home/aish/.aish/models'
```

The volume survives container removal. Removing `aish-home` permanently deletes
the model, config, and command history, so do that only when a complete reset is
intended.

## Troubleshooting

`model download failed`: verify Docker has internet access and rerun the download
command.

`failed to start AiSH model runtime`: verify `llama-cli`:

```bash
docker run --rm --entrypoint llama-cli aish-beta-llama --version
```

`Permission denied` under `/home/aish/.aish`: repair volume ownership:

```bash
docker run --rm --user root \
  -v aish-home:/home/aish/.aish \
  --entrypoint /bin/bash \
  aish-beta-llama \
  -lc 'chown -R aish:aish /home/aish/.aish'
```
