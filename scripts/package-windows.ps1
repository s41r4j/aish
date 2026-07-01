param(
    [string]$LlamaCppRef = "master",
    [string]$BundleName = "aish-windows-x86_64"
)

$ErrorActionPreference = "Stop"

$ProjectDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$DistDir = Join-Path $ProjectDir "dist"
$BuildDir = Join-Path $ProjectDir ".build"
$LlamaDir = Join-Path $BuildDir "llama.cpp"
$BundleDir = Join-Path $DistDir $BundleName
$Archive = Join-Path $DistDir "$BundleName.zip"

Set-Location $ProjectDir

cargo build --release

if (!(Test-Path $LlamaDir)) {
    New-Item -ItemType Directory -Force -Path $BuildDir | Out-Null
    git clone --depth 1 --branch $LlamaCppRef https://github.com/ggml-org/llama.cpp.git $LlamaDir
}

$LlamaBuildDir = Join-Path $LlamaDir "build"
cmake -S $LlamaDir -B $LlamaBuildDir -DBUILD_SHARED_LIBS=ON -DLLAMA_CURL=OFF -DGGML_NATIVE=OFF
cmake --build $LlamaBuildDir --config Release --target llama-cli

if (Test-Path $BundleDir) {
    Remove-Item -Recurse -Force $BundleDir
}
if (Test-Path $Archive) {
    Remove-Item -Force $Archive
}

New-Item -ItemType Directory -Force -Path (Join-Path $BundleDir "bin") | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $BundleDir "runtime") | Out-Null

Copy-Item (Join-Path $ProjectDir "target\release\aish.exe") (Join-Path $BundleDir "bin\aish.exe")

$LlamaBinRoots = @(
    (Join-Path $LlamaBuildDir "bin\Release"),
    (Join-Path $LlamaBuildDir "bin")
)

$Copied = $false
foreach ($Root in $LlamaBinRoots) {
    $Cli = Join-Path $Root "llama-cli.exe"
    if (Test-Path $Cli) {
        Copy-Item (Join-Path $Root "*") (Join-Path $BundleDir "runtime") -Recurse -Force
        $Copied = $true
        break
    }
}

if (!$Copied) {
    throw "Could not find built llama-cli.exe under $LlamaBuildDir"
}

@'
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
'@ | Set-Content -Path (Join-Path $BundleDir "install.ps1")

Compress-Archive -Path $BundleDir -DestinationPath $Archive
Write-Host "Built $Archive"
