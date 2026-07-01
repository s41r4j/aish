#!/usr/bin/env sh
set -eu

bundle_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
install_root="${AISH_INSTALL_ROOT:-$HOME/.local/share/aish}"
bin_dir="${AISH_BIN_DIR:-$HOME/.local/bin}"

mkdir -p "$install_root" "$bin_dir"
cp -R "$bundle_dir/bin" "$install_root/"
cp -R "$bundle_dir/runtime" "$install_root/"
ln -sf "$install_root/bin/aish" "$bin_dir/aish"

printf 'Installed AiSH at %s/aish\n' "$bin_dir"
case ":${PATH:-}:" in
  *":$bin_dir:"*) ;;
  *) printf 'Add %s to PATH, then run: aish\n' "$bin_dir" ;;
esac
