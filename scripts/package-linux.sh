#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
dist_dir="$project_dir/dist"
bundle_name="${AISH_BUNDLE_NAME:-aish-linux-x86_64}"
bundle_dir="$dist_dir/$bundle_name"
archive="$dist_dir/$bundle_name.tar.gz"
image="${AISH_BUNDLE_IMAGE:-aish-bundle-linux-x86_64}"
platform="${AISH_DOCKER_PLATFORM:-linux/amd64}"

cd "$project_dir"

docker build --platform "$platform" -t "$image" .

container=$(docker create "$image")
cleanup() {
  docker rm -f "$container" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

rm -rf "$bundle_dir" "$archive"
mkdir -p "$bundle_dir/bin" "$bundle_dir/runtime"

docker cp "$container:/usr/local/bin/aish" "$bundle_dir/bin/aish"
docker cp "$container:/usr/local/llama/bin/." "$bundle_dir/runtime/"
docker cp "$container:/usr/local/bin/aish-download-model" "$bundle_dir/bin/aish-download-model"
docker cp "$container:/lib/x86_64-linux-gnu/libgomp.so.1" "$bundle_dir/runtime/libgomp.so.1"
docker cp "$container:/lib/x86_64-linux-gnu/libgomp.so.1.0.0" "$bundle_dir/runtime/libgomp.so.1.0.0"

chmod +x "$bundle_dir/bin/aish" "$bundle_dir/bin/aish-download-model"
if [ -f "$bundle_dir/runtime/llama-cli" ]; then
  chmod +x "$bundle_dir/runtime/llama-cli"
fi

cat > "$bundle_dir/install.sh" <<'EOF'
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
EOF
chmod +x "$bundle_dir/install.sh"

tar -czf "$archive" -C "$dist_dir" "$bundle_name"

printf 'Built %s\n' "$archive"
