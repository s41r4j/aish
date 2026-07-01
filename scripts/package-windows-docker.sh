#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
dist_dir="$project_dir/dist"
bundle_name="${AISH_BUNDLE_NAME:-aish-windows-x86_64}"
bundle_dir="$dist_dir/$bundle_name"
archive="$dist_dir/$bundle_name.zip"
image="${AISH_BUNDLE_IMAGE:-aish-bundle-windows-x86_64}"

cd "$project_dir"

docker build -f Dockerfile.windows -t "$image" .

container=$(docker create "$image")
cleanup() {
  docker rm -f "$container" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

rm -rf "$bundle_dir" "$archive"
mkdir -p "$bundle_dir"

docker cp "$container:/out/." "$bundle_dir/"

cat > "$bundle_dir/install.ps1" <<'EOF'
param(
    [string]$InstallRoot = "$env:LOCALAPPDATA\AiSH",
    [string]$BinDir = "$env:USERPROFILE\.local\bin"
)

$ErrorActionPreference = "Stop"
$BundleDir = Split-Path -Parent $MyInvocation.MyCommand.Path

New-Item -ItemType Directory -Force -Path $InstallRoot | Out-Null
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item (Join-Path $BundleDir "bin") $InstallRoot -Recurse -Force
Copy-Item (Join-Path $BundleDir "runtime") $InstallRoot -Recurse -Force

$CmdPath = Join-Path $BinDir "aish.cmd"
$AishExe = Join-Path $InstallRoot "bin\aish.exe"
Set-Content -Path $CmdPath -Value "@echo off`r`n`"$AishExe`" %*`r`n"

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if (($UserPath -split ";") -notcontains $BinDir) {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "Added $BinDir to the user PATH. Open a new terminal before running aish."
}

Write-Host "Installed AiSH at $AishExe"
EOF

if command -v zip >/dev/null 2>&1; then
  (cd "$dist_dir" && zip -qr "$archive" "$bundle_name")
else
  docker run --rm -v "$dist_dir:/dist" debian:bookworm-slim \
    /bin/sh -lc "apt-get update >/dev/null && apt-get install -y zip >/dev/null && cd /dist && zip -qr '$bundle_name.zip' '$bundle_name'"
fi

printf 'Built %s\n' "$archive"
