#Requires -Version 5.1
<#
.SYNOPSIS
    Installs vibestats on Windows.

.DESCRIPTION
    Downloads the latest native Windows vibestats release artifact, verifies the
    SHA256 checksum, installs vibestats.exe to the current user's local Programs
    folder, optionally installs/authenticates GitHub CLI, adds vibestats to the
    user PATH, and runs `vibestats auth`.

.PARAMETER Repo
    GitHub repository that owns the vibestats release artifacts.

.PARAMETER Version
    Release version to install, such as v2.4.1. Defaults to latest.

.PARAMETER InstallDir
    Directory where vibestats.exe will be installed.

.PARAMETER SkipGhInstall
    Do not attempt to install GitHub CLI if gh.exe is missing.

.PARAMETER SkipGhAuth
    Do not run `gh auth status` / `gh auth login`.

.PARAMETER SkipPathUpdate
    Do not add the install directory to the current user's PATH.

.PARAMETER SkipVibestatsAuth
    Do not run `vibestats auth` after installing the binary.

.EXAMPLE
    irm https://vibestats.dev/install.ps1 | iex

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version v2.4.1
#>
[CmdletBinding()]
param(
    [string]$Repo = "stephenleo/vibestats",
    [string]$Version = "latest",
    [string]$InstallDir = $(Join-Path $env:LOCALAPPDATA "Programs\vibestats"),
    [switch]$SkipGhInstall,
    [switch]$SkipGhAuth,
    [switch]$SkipPathUpdate,
    [switch]$SkipVibestatsAuth,
    [switch]$Help
)

Set-StrictMode -Version 2.0
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "=== $Message ===" -ForegroundColor Cyan
}

function Write-Info {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

function Fail {
    param([string]$Message)
    Write-Host ""
    Write-Host $Message -ForegroundColor Red
    exit 1
}

function Show-Usage {
    Write-Host @"
Usage:
  powershell -ExecutionPolicy Bypass -File .\install.ps1 [options]

Options:
  -Repo <owner/repo>       Release repository. Default: stephenleo/vibestats
  -Version <tag|latest>    Release tag to install. Default: latest
  -InstallDir <path>       Install directory. Default: %LOCALAPPDATA%\Programs\vibestats
  -SkipGhInstall           Do not install GitHub CLI if missing
  -SkipGhAuth              Do not run gh auth checks/login
  -SkipPathUpdate          Do not add InstallDir to user PATH
  -SkipVibestatsAuth       Do not run vibestats auth after install
  -Help                    Show this help

Examples:
  powershell -ExecutionPolicy Bypass -File .\install.ps1
  powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version v2.4.1
"@
}

function Get-GhPath {
    $cmd = Get-Command gh.exe -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $knownPaths = @(
        (Join-Path $env:ProgramFiles "GitHub CLI\gh.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "GitHub CLI\gh.exe"),
        (Join-Path $env:LOCALAPPDATA "Programs\GitHub CLI\gh.exe"),
        (Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Links\gh.exe")
    )

    foreach ($path in $knownPaths) {
        if ($path -and (Test-Path -LiteralPath $path)) {
            return $path
        }
    }

    return $null
}

function Install-GhIfMissing {
    $gh = Get-GhPath
    if ($gh) {
        Write-Success "GitHub CLI found: $gh"
        return $gh
    }

    if ($SkipGhInstall) {
        Fail "GitHub CLI was not found and -SkipGhInstall was set. Install GitHub CLI, then rerun this script."
    }

    $winget = Get-Command winget.exe -ErrorAction SilentlyContinue
    if (-not $winget) {
        Fail "GitHub CLI was not found, and winget.exe is not available. Install GitHub CLI manually, then rerun this script."
    }

    Write-Info "GitHub CLI not found. Installing with winget..."
    & $winget.Source install --id GitHub.cli -e --source winget --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE -ne 0) {
        Fail "winget failed to install GitHub CLI. Exit code: $LASTEXITCODE"
    }

    $gh = Get-GhPath
    if (-not $gh) {
        Fail "GitHub CLI install appeared to complete, but gh.exe was not found. Open a new terminal and rerun this script."
    }

    Write-Success "GitHub CLI installed: $gh"
    return $gh
}

function Assert-GhVersion {
    param([string]$GhPath)

    $versionOutput = & $GhPath --version
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to run gh --version."
    }

    $firstLine = ($versionOutput | Select-Object -First 1)
    Write-Info $firstLine

    $match = [regex]::Match($firstLine, '\d+(\.\d+)+')
    if (-not $match.Success) {
        Write-Warn "Could not parse gh version. Continuing anyway."
        return
    }

    $version = [version]$match.Value
    if ($version.Major -lt 2) {
        Fail "GitHub CLI 2.0 or newer is required. Found: $version"
    }
}

function Ensure-GhAuth {
    param([string]$GhPath)

    if ($SkipGhAuth) {
        Write-Warn "Skipping GitHub CLI authentication check."
        return
    }

    & $GhPath auth status *> $null
    if ($LASTEXITCODE -eq 0) {
        Write-Success "GitHub CLI is authenticated."
        return
    }

    Write-Info "GitHub CLI is not authenticated. Starting gh auth login..."
    & $GhPath auth login
    if ($LASTEXITCODE -ne 0) {
        Fail "gh auth login failed. Exit code: $LASTEXITCODE"
    }

    & $GhPath auth status *> $null
    if ($LASTEXITCODE -ne 0) {
        Fail "GitHub CLI still does not appear authenticated after gh auth login."
    }

    Write-Success "GitHub CLI authentication complete."
}

function Get-WindowsTarget {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64" -or $arch -eq "x86_64") {
        return "x86_64-pc-windows-msvc"
    }

    Fail "Unsupported Windows architecture: $arch. This installer currently supports x64 Windows only."
}

function Get-ReleaseBaseUrl {
    param(
        [string]$RepoName,
        [string]$ReleaseVersion
    )

    if ($ReleaseVersion -eq "latest") {
        return "https://github.com/$RepoName/releases/latest/download"
    }

    return "https://github.com/$RepoName/releases/download/$ReleaseVersion"
}

function Download-File {
    param(
        [string]$Uri,
        [string]$OutFile
    )

    Write-Info "Downloading $Uri"
    Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $OutFile
}

function Assert-Checksum {
    param(
        [string]$FilePath,
        [string]$ChecksumPath
    )

    $checksumText = Get-Content -LiteralPath $ChecksumPath -Raw
    $match = [regex]::Match($checksumText, '(?i)\b[a-f0-9]{64}\b')
    if (-not $match.Success) {
        Fail "Could not find a SHA256 hash in checksum file: $ChecksumPath"
    }

    $expected = $match.Value.ToLowerInvariant()
    $actual = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLowerInvariant()

    if ($actual -ne $expected) {
        Fail "Checksum mismatch for $FilePath.`nExpected: $expected`nActual:   $actual"
    }

    Write-Success "Checksum verified: $actual"
}

function Add-UserPathEntry {
    param([string]$Directory)

    if ($SkipPathUpdate) {
        Write-Warn "Skipping user PATH update."
        return
    }

    $resolved = [System.IO.Path]::GetFullPath($Directory).TrimEnd('\')
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) {
        $userPath = ""
    }

    $entries = @()
    if ($userPath.Trim()) {
        $entries = $userPath -split ';' | Where-Object { $_ -and $_.Trim() }
    }

    $alreadyPresent = $false
    foreach ($entry in $entries) {
        $normalized = [System.IO.Path]::GetFullPath($entry).TrimEnd('\')
        if ([string]::Equals($normalized, $resolved, [System.StringComparison]::OrdinalIgnoreCase)) {
            $alreadyPresent = $true
            break
        }
    }

    if (-not $alreadyPresent) {
        $newPath = if ($userPath.Trim()) { "$userPath;$resolved" } else { $resolved }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Success "Added to user PATH: $resolved"
    } else {
        Write-Success "Install directory already exists in user PATH."
    }

    $processPath = [Environment]::GetEnvironmentVariable("Path", "Process")
    $processEntries = @()
    if ($processPath) {
        $processEntries = $processPath -split ';' | Where-Object { $_ -and $_.Trim() }
    }

    $processAlreadyPresent = $false
    foreach ($entry in $processEntries) {
        $normalized = [System.IO.Path]::GetFullPath($entry).TrimEnd('\')
        if ([string]::Equals($normalized, $resolved, [System.StringComparison]::OrdinalIgnoreCase)) {
            $processAlreadyPresent = $true
            break
        }
    }

    if (-not $processAlreadyPresent) {
        $env:Path = "$resolved;$env:Path"
    }
}

function Install-Vibestats {
    if (-not $env:LOCALAPPDATA) {
        Fail "LOCALAPPDATA is not set. This installer needs a normal user profile."
    }

    $target = Get-WindowsTarget
    $baseUrl = Get-ReleaseBaseUrl -RepoName $Repo -ReleaseVersion $Version
    $archive = "vibestats-$target.zip"
    $checksum = "$archive.sha256"

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("vibestats-install-" + [guid]::NewGuid().ToString("N"))
    $extractDir = Join-Path $tempDir "extract"
    New-Item -ItemType Directory -Force -Path $tempDir, $extractDir | Out-Null

    try {
        $zipPath = Join-Path $tempDir $archive
        $checksumPath = Join-Path $tempDir $checksum

        Download-File -Uri "$baseUrl/$archive" -OutFile $zipPath
        Download-File -Uri "$baseUrl/$checksum" -OutFile $checksumPath

        Write-Step "Verify checksum"
        Assert-Checksum -FilePath $zipPath -ChecksumPath $checksumPath

        Write-Step "Extract archive"
        Expand-Archive -LiteralPath $zipPath -DestinationPath $extractDir -Force

        $exe = Get-ChildItem -LiteralPath $extractDir -Filter "vibestats.exe" -Recurse | Select-Object -First 1
        if (-not $exe) {
            Fail "Archive did not contain vibestats.exe."
        }

        Write-Step "Install vibestats.exe"
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        $dest = Join-Path $InstallDir "vibestats.exe"
        Copy-Item -LiteralPath $exe.FullName -Destination $dest -Force
        Unblock-File -LiteralPath $dest -ErrorAction SilentlyContinue

        & $dest --version
        if ($LASTEXITCODE -ne 0) {
            Fail "Installed vibestats.exe, but --version failed."
        }

        Add-UserPathEntry -Directory $InstallDir

        if (-not $SkipVibestatsAuth) {
            Write-Step "Run vibestats auth"
            & $dest auth
            if ($LASTEXITCODE -ne 0) {
                Write-Warn "vibestats auth exited with code $LASTEXITCODE. You can retry later with: vibestats auth"
            }
        } else {
            Write-Warn "Skipping vibestats auth."
        }

        Write-Success "vibestats installed successfully: $dest"
        Write-Host ""
        Write-Host "Open a new terminal, then run:" -ForegroundColor Cyan
        Write-Host "  vibestats --help"
        Write-Host "  vibestats status"
        Write-Host ""
        Write-Warn "Note: this Windows installer installs the native binary and runs auth. Full first-install setup parity with install.sh is still being ported."
    }
    finally {
        Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($Help) {
    Show-Usage
    return
}

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
} catch {
    # Best effort for older PowerShell hosts.
}

Write-Host "=== vibestats Windows installer ===" -ForegroundColor Cyan

Write-Step "GitHub CLI"
$ghPath = Install-GhIfMissing
Assert-GhVersion -GhPath $ghPath
Ensure-GhAuth -GhPath $ghPath

Write-Step "Install vibestats"
Install-Vibestats

Write-Step "Complete"
Write-Success "Installation complete."
