#!/usr/bin/env sh
set -eu

repo="${AISH_MODEL_REPO:-s41r4j/aish-qwen25-coder-1.5b-gguf-q4km-200}"
file="${AISH_MODEL_FILE:-aish.gguf}"
home_dir="${AISH_HOME:-$HOME/.aish}"
target_dir="$home_dir/models"
target="$target_dir/aish.gguf"

if [ -z "${HF_TOKEN:-}" ]; then
  echo "HF_TOKEN is required for private model downloads" >&2
  exit 1
fi

mkdir -p "$target_dir"

url="https://huggingface.co/$repo/resolve/main/$file"
echo "Downloading $url"
curl -L \
  -H "Authorization: Bearer $HF_TOKEN" \
  -o "$target" \
  "$url"

echo "Saved $target"
