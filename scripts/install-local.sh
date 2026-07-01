#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
install_dir="${AISH_INSTALL_DIR:-$HOME/.local/bin}"
share_dir="${AISH_SHARE_DIR:-$HOME/.local/share/aish}"

cd "$project_dir"
cargo build --release

mkdir -p "$install_dir" "$share_dir/bin" "$share_dir/runtime"
cp target/release/aish "$share_dir/bin/aish"
chmod +x "$share_dir/bin/aish"
ln -sf "$share_dir/bin/aish" "$install_dir/aish"

if [ -f "$project_dir/runtime/llama-cli" ]; then
  cp "$project_dir/runtime/llama-cli" "$share_dir/runtime/llama-cli"
  chmod +x "$share_dir/runtime/llama-cli"
fi

printf 'Installed AiSH at %s/aish\n' "$install_dir"
case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *) printf 'Add %s to PATH, then run: aish\n' "$install_dir" ;;
esac
