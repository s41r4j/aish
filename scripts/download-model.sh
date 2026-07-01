#!/usr/bin/env sh
set -eu

repo="${AISH_MODEL_REPO:-Qwen/Qwen2.5-Coder-1.5B-Instruct-GGUF}"
file="${AISH_MODEL_FILE:-qwen2.5-coder-1.5b-instruct-q2_k.gguf}"
home_dir="${AISH_HOME:-$HOME/.aish}"
target_dir="$home_dir/models"
target="${AISH_MODEL_PATH:-$target_dir/aish.gguf}"

mkdir -p "$(dirname "$target")"

url="https://huggingface.co/$repo/resolve/main/$file"
echo "Downloading $url"
if [ -n "${HF_TOKEN:-}" ]; then
  curl -L --fail --progress-bar \
    -H "Authorization: Bearer $HF_TOKEN" \
    -o "$target" \
    "$url"
else
  curl -L --fail --progress-bar \
    -o "$target" \
    "$url"
fi

echo "Saved $target"
