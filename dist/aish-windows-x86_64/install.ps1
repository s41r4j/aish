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
