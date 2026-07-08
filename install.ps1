#Requires -Version 5.1
<#
.SYNOPSIS
    Install vibestats on Windows.

.DESCRIPTION
    Downloads the native Windows vibestats release artifact from GitHub Releases,
    verifies its SHA256 checksum, installs vibestats.exe to the current user's
    local Programs folder, ensures GitHub CLI is available/authenticated, adds
    vibestats to the user PATH, bootstraps the vibestats-data repository and
    local config, installs the aggregate workflow and repository secrets, runs
    vibestats auth, configures Claude Code and Codex hooks when their settings
    directories already exist, injects profile README markers, runs an initial
    backfill, and dispatches the aggregate workflow.

.PARAMETER Yes
    Non-interactive mode: accept installer defaults automatically where possible.

.PARAMETER Repo
    GitHub repository that owns the vibestats release artifacts.

.PARAMETER Version
    Release version to install, such as v2.4.1. Defaults to latest.

.PARAMETER InstallDir
    Directory where vibestats.exe will be installed.

.PARAMETER SkipGhInstall
    Do not attempt to install GitHub CLI if gh.exe is missing.

.PARAMETER SkipGhAuth
    Do not run gh auth status / gh auth login.

.PARAMETER SkipPathUpdate
    Do not add the install directory to the current user's PATH.

.PARAMETER SkipVibestatsAuth
    Do not run vibestats auth after installing the binary.

.PARAMETER SkipHookConfiguration
    Do not configure Claude Code or Codex hooks.

.PARAMETER SkipReadmeMarkers
    Do not add vibestats markers to the GitHub profile README.

.PARAMETER SkipBackfill
    Do not run vibestats sync --backfill after installing.

.PARAMETER SkipInitialWorkflowDispatch
    Do not dispatch the aggregate workflow after installing.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version v2.4.1

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\install.ps1 -Yes
#>
[CmdletBinding()]
param(
    [switch]$Yes,
    [string]$Repo = "stephenleo/vibestats",
    [string]$Version = "latest",
    [string]$InstallDir = $(Join-Path $env:LOCALAPPDATA "Programs\vibestats"),
    [switch]$SkipGhInstall,
    [switch]$SkipGhAuth,
    [switch]$SkipPathUpdate,
    [switch]$SkipVibestatsAuth,
    [switch]$SkipHookConfiguration,
    [switch]$SkipReadmeMarkers,
    [switch]$SkipBackfill,
    [switch]$SkipInitialWorkflowDispatch,
    [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"
$Utf8NoBom = New-Object System.Text.UTF8Encoding $false

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
  -Yes                     Non-interactive mode; accept defaults where possible
  -Repo <owner/repo>       Release repository. Default: stephenleo/vibestats
  -Version <tag|latest>    Release tag to install. Default: latest
  -InstallDir <path>       Install directory. Default: %LOCALAPPDATA%\Programs\vibestats
  -SkipGhInstall           Do not install GitHub CLI if missing
  -SkipGhAuth              Do not run gh auth checks/login
  -SkipPathUpdate          Do not add InstallDir to user PATH
  -SkipVibestatsAuth       Do not run vibestats auth after install
  -SkipHookConfiguration   Do not configure Claude Code or Codex hooks
  -SkipReadmeMarkers       Do not add profile README markers
  -SkipBackfill            Do not run vibestats sync --backfill
  -SkipInitialWorkflowDispatch
                           Do not dispatch the aggregate workflow
  -Help                    Show this help

Examples:
  powershell -ExecutionPolicy Bypass -File .\install.ps1
  powershell -ExecutionPolicy Bypass -File .\install.ps1 -Version v2.4.1
  powershell -ExecutionPolicy Bypass -File .\install.ps1 -Yes
"@
}

function Test-Interactive {
    return ([Environment]::UserInteractive -and -not [Console]::IsInputRedirected)
}

function Get-NativeTarget {
    $arch = $env:PROCESSOR_ARCHITECTURE

    # WOW64: 32-bit PowerShell on 64-bit Windows reports x86.
    if ($arch -eq "x86" -and $env:PROCESSOR_ARCHITEW6432) {
        $arch = $env:PROCESSOR_ARCHITEW6432
    }

    if ($arch -eq "AMD64" -or $arch -eq "x86_64") {
        return "x86_64-pc-windows-msvc"
    }

    Fail "Unsupported Windows architecture: $arch. This installer currently supports x64 Windows only."
}

function Get-GhPath {
    $cmd = Get-Command gh.exe -ErrorAction SilentlyContinue
    if ($cmd) {
        return $cmd.Source
    }

    $knownPaths = @()

    if ($env:ProgramFiles) {
        $knownPaths += (Join-Path $env:ProgramFiles "GitHub CLI\gh.exe")
    }

    if (${env:ProgramFiles(x86)}) {
        $knownPaths += (Join-Path ${env:ProgramFiles(x86)} "GitHub CLI\gh.exe")
    }

    if ($env:LOCALAPPDATA) {
        $knownPaths += (Join-Path $env:LOCALAPPDATA "Programs\GitHub CLI\gh.exe")
        $knownPaths += (Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Links\gh.exe")
    }

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

    $versionValue = [version]$match.Value
    if ($versionValue.Major -lt 2) {
        Fail "GitHub CLI 2.0 or newer is required. Found: $versionValue"
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

    if (-not (Test-Interactive)) {
        Fail "GitHub CLI is not authenticated and this shell is non-interactive. Run 'gh auth login' first, or rerun with -SkipGhAuth."
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

function Invoke-GhJson {
    param(
        [string]$GhPath,
        [string[]]$Arguments
    )

    $output = & $GhPath @Arguments
    if ($LASTEXITCODE -ne 0) {
        Fail "gh $($Arguments -join ' ') failed with exit code $LASTEXITCODE."
    }

    $jsonText = ($output | Out-String).Trim()
    if (-not $jsonText) {
        return $null
    }

    return $jsonText | ConvertFrom-Json
}

function Get-GithubLogin {
    param([string]$GhPath)

    $user = Invoke-GhJson -GhPath $GhPath -Arguments @("api", "/user")
    if (-not $user.login) {
        Fail "Could not determine GitHub username from gh api /user."
    }

    return [string]$user.login
}

function Test-GithubRepoExists {
    param(
        [string]$GhPath,
        [string]$RepoName
    )

    & $GhPath repo view $RepoName --json name *> $null
    return ($LASTEXITCODE -eq 0)
}

function Ensure-VibestatsDataRepo {
    param(
        [string]$GhPath,
        [string]$GitHubUser
    )

    $repoName = "$GitHubUser/vibestats-data"

    if (Test-GithubRepoExists -GhPath $GhPath -RepoName $repoName) {
        Write-Success "Existing repository detected: $repoName"
        return $repoName
    }

    Write-Info "Creating private repository: $repoName"
    & $GhPath repo create $repoName --private --description "Private VibeStats data repository"
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to create repository: $repoName"
    }

    Write-Success "Repository created: $repoName"
    return $repoName
}

function ConvertTo-Slug {
    param([string]$Value)

    $slug = ($Value.ToLowerInvariant() -replace '[^a-z0-9]+', '-').Trim('-')
    if (-not $slug) {
        $slug = "machine"
    }

    if ($slug.Length -gt 20) {
        $slug = $slug.Substring(0, 20).Trim('-')
    }

    if (-not $slug) {
        $slug = "machine"
    }

    return $slug
}

function Get-MachineSuffix {
    try {
        $machineGuid = (Get-ItemProperty -LiteralPath "HKLM:\SOFTWARE\Microsoft\Cryptography" -Name MachineGuid -ErrorAction Stop).MachineGuid
        $clean = ($machineGuid -replace '[^a-fA-F0-9]', '').ToLowerInvariant()
        if ($clean.Length -ge 6) {
            return $clean.Substring(0, 6)
        }
    }
    catch {
        # Best effort; fall through to generated suffix.
    }

    return ([guid]::NewGuid().ToString("N").Substring(0, 6)).ToLowerInvariant()
}

function New-VibestatsMachineId {
    $hostName = [System.Net.Dns]::GetHostName()
    $slug = ConvertTo-Slug -Value $hostName
    $suffix = Get-MachineSuffix

    return "$slug-$suffix"
}

function Get-IsoUtcNow {
    return [DateTime]::UtcNow.ToString("yyyy-MM-ddTHH:mm:ssZ")
}

function Get-VibestatsConfigPath {
    if (-not $env:APPDATA) {
        Fail "APPDATA is not set. Cannot write vibestats config.toml."
    }

    return (Join-Path $env:APPDATA "vibestats\config.toml")
}

function Write-VibestatsConfig {
    param(
        [string]$GhPath,
        [string]$MachineId,
        [string]$RepoName
    )

    $tokenOutput = & $GhPath auth token
    if ($LASTEXITCODE -ne 0) {
        Fail "Could not read GitHub token from gh auth token."
    }

    $token = (($tokenOutput | Out-String).Trim())
    if (-not $token) {
        Fail "gh auth token returned an empty token."
    }

    $configPath = Get-VibestatsConfigPath
    $configDir = Split-Path -Parent $configPath
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null

    $config = @"
oauth_token = "$token"
machine_id = "$MachineId"
vibestats_data_repo = "$RepoName"
"@

    [System.IO.File]::WriteAllText($configPath, $config, $Utf8NoBom)
    Write-Success "Wrote config: $configPath"

    return $configPath
}

function Decode-GitHubContent {
    param([string]$EncodedContent)

    $normalized = $EncodedContent -replace '\s', ''
    $bytes = [Convert]::FromBase64String($normalized)
    return [System.Text.Encoding]::UTF8.GetString($bytes)
}

function Encode-GitHubContent {
    param([string]$Content)

    $bytes = [System.Text.Encoding]::UTF8.GetBytes($Content)
    return [Convert]::ToBase64String($bytes)
}

function Get-RegistryDocument {
    param(
        [string]$GhPath,
        [string]$RepoName
    )

    $responseText = & $GhPath api "repos/$RepoName/contents/registry.json" 2>$null
    if ($LASTEXITCODE -ne 0) {
        return @{
            Sha = ""
            Document = @{
                machines = @()
            }
        }
    }

    $response = (($responseText | Out-String).Trim()) | ConvertFrom-Json
    $content = Decode-GitHubContent -EncodedContent $response.content

    try {
        $document = ConvertTo-Hashtable ($content | ConvertFrom-Json)
    }
    catch {
        Write-Warn "registry.json exists but could not be parsed. Replacing it with a fresh registry."
        $document = @{
            machines = @()
        }
    }

    if (-not ($document -is [hashtable])) {
        $document = @{
            machines = @()
        }
    }

    if (-not $document.ContainsKey("machines") -or -not $document["machines"]) {
        $document["machines"] = @()
    }

    return @{
        Sha = [string]$response.sha
        Document = $document
    }
}

function Save-RegistryDocument {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$Sha,
        [hashtable]$Document,
        [string]$Message
    )

    $json = $Document | ConvertTo-Json -Depth 100
    $encoded = Encode-GitHubContent -Content ($json + [Environment]::NewLine)

    $args = @(
        "api",
        "repos/$RepoName/contents/registry.json",
        "--method",
        "PUT",
        "--field",
        "message=$Message",
        "--field",
        "content=$encoded"
    )

    if ($Sha) {
        $args += @("--field", "sha=$Sha")
    }

    & $GhPath @args *> $null
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to update registry.json in $RepoName."
    }
}

function Register-VibestatsMachine {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$MachineId
    )

    Write-Info "Registering machine in $RepoName/registry.json..."

    $registry = Get-RegistryDocument -GhPath $GhPath -RepoName $RepoName
    $document = $registry.Document
    $machines = @($document["machines"])
    $hostName = [System.Net.Dns]::GetHostName()
    $timestamp = Get-IsoUtcNow

    $found = $false
    for ($i = 0; $i -lt $machines.Count; $i++) {
        $machine = ConvertTo-Hashtable $machines[$i]
        if ($machine["machine_id"] -eq $MachineId) {
            $machine["hostname"] = $hostName
            $machine["status"] = "active"
            $machine["last_seen"] = $timestamp
            $machines[$i] = $machine
            $found = $true
            break
        }
    }

    if (-not $found) {
        $machines += @{
            machine_id = $MachineId
            hostname = $hostName
            status = "active"
            last_seen = $timestamp
        }
    }

    $document["machines"] = @($machines)

    Save-RegistryDocument `
        -GhPath $GhPath `
        -RepoName $RepoName `
        -Sha $registry.Sha `
        -Document $document `
        -Message "chore: register machine $MachineId"

    Write-Success "Machine registered: $MachineId"
}

function Initialize-VibestatsConfig {
    param([string]$GhPath)

    $gitHubUser = Get-GithubLogin -GhPath $GhPath
    $repoName = Ensure-VibestatsDataRepo -GhPath $GhPath -GitHubUser $gitHubUser
    $machineId = New-VibestatsMachineId

    Register-VibestatsMachine -GhPath $GhPath -RepoName $repoName -MachineId $machineId
    Write-VibestatsConfig -GhPath $GhPath -MachineId $machineId -RepoName $repoName | Out-Null

    return @{
        GitHubUser = $gitHubUser
        RepoName = $repoName
        MachineId = $machineId
    }
}

function Get-AggregateWorkflowContent {
    return @'
# aggregate.yml - Copy this file to your vibestats-data/.github/workflows/ directory.
# It runs the vibestats community action daily to aggregate your AI coding session data
# and update your GitHub profile heatmap automatically.
name: Aggregate vibestats data

on:
  schedule:
    - cron: '0 2 * * *'   # Daily at 02:00 UTC
  workflow_dispatch:        # Allow manual runs

jobs:
  aggregate:
    runs-on: ubuntu-latest
    permissions:
      contents: write   # lets the action commit the GitHub contributions snapshot back to vibestats-data
    steps:
      - uses: stephenleo/vibestats@v2
        with:
          token: ${{ secrets.VIBESTATS_TOKEN }}
          github-token: ${{ secrets.VIBESTATS_GH_TOKEN }}
          profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}
'@
}

function Get-GithubFileSha {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$Path
    )

    $responseText = & $GhPath api "repos/$RepoName/contents/$Path" 2>$null
    if ($LASTEXITCODE -ne 0) {
        return ""
    }

    try {
        $response = (($responseText | Out-String).Trim()) | ConvertFrom-Json
        if ($response.sha) {
            return [string]$response.sha
        }
    }
    catch {
        return ""
    }

    return ""
}

function Write-GithubFile {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$Path,
        [string]$Content,
        [string]$AddMessage,
        [string]$UpdateMessage
    )

    $sha = Get-GithubFileSha -GhPath $GhPath -RepoName $RepoName -Path $Path
    $encoded = Encode-GitHubContent -Content $Content

    $message = if ($sha) { $UpdateMessage } else { $AddMessage }

    $ghArgs = @(
        "api",
        "repos/$RepoName/contents/$Path",
        "--method",
        "PUT",
        "--field",
        "message=$message",
        "--field",
        "content=$encoded"
    )

    if ($sha) {
        $ghArgs += @("--field", "sha=$sha")
    }

    & $GhPath @ghArgs *> $null
    if ($LASTEXITCODE -ne 0) {
        Fail "Failed to write $Path in $RepoName."
    }
}

function Write-AggregateWorkflow {
    param(
        [string]$GhPath,
        [string]$RepoName
    )

    Write-Info "Writing aggregate workflow to $RepoName..."
    $workflowContent = Get-AggregateWorkflowContent

    Write-GithubFile `
        -GhPath $GhPath `
        -RepoName $RepoName `
        -Path ".github/workflows/aggregate.yml" `
        -Content ($workflowContent + [Environment]::NewLine) `
        -AddMessage "Add vibestats aggregate workflow" `
        -UpdateMessage "Update vibestats aggregate workflow"

    Write-Success "Workflow written: $RepoName/.github/workflows/aggregate.yml"
}

function Set-GhSecretValue {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$SecretName,
        [string]$SecretValue,
        [switch]$NonFatal
    )

    if (-not $SecretValue) {
        if ($NonFatal) {
            Write-Warn "$SecretName was empty; skipping."
            return $false
        }

        Fail "$SecretName was empty."
    }

    $SecretValue | & $GhPath secret set $SecretName --repo $RepoName --body-file - *> $null
    if ($LASTEXITCODE -ne 0) {
        if ($NonFatal) {
            Write-Warn "Could not set $SecretName."
            return $false
        }

        Fail "Failed to set $SecretName."
    }

    Write-Success "$SecretName secret set successfully."
    return $true
}

function Get-GhAuthToken {
    param([string]$GhPath)

    $tokenOutput = & $GhPath auth token
    if ($LASTEXITCODE -ne 0) {
        return ""
    }

    return (($tokenOutput | Out-String).Trim())
}

function New-FineGrainedVibestatsToken {
    param(
        [string]$GhPath,
        [string]$GitHubUser
    )

    $year = [DateTime]::UtcNow.Year

    $tokenOutput = & $GhPath api /user/personal_access_tokens `
        --method POST `
        --field "name=vibestats-$year" `
        --field "expiration=never" `
        --field "repositories=[`"$GitHubUser`"]" `
        --field 'permissions={"contents":"write"}' `
        --jq ".token" 2>$null

    if ($LASTEXITCODE -ne 0) {
        return ""
    }

    return (($tokenOutput | Out-String).Trim())
}

function Set-VibestatsTokenSecret {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$GitHubUser
    )

    Write-Info "Setting VIBESTATS_TOKEN Actions secret..."

    $fineGrainedToken = New-FineGrainedVibestatsToken -GhPath $GhPath -GitHubUser $GitHubUser
    if ($fineGrainedToken) {
        if (Set-GhSecretValue -GhPath $GhPath -RepoName $RepoName -SecretName "VIBESTATS_TOKEN" -SecretValue $fineGrainedToken -NonFatal) {
            return
        }
    }

    Write-Warn "Fine-grained PAT creation failed or was blocked. Falling back to gh auth token."
    $fallbackToken = Get-GhAuthToken -GhPath $GhPath
    Set-GhSecretValue -GhPath $GhPath -RepoName $RepoName -SecretName "VIBESTATS_TOKEN" -SecretValue $fallbackToken | Out-Null
}

function Set-GithubContributionsTokenSecret {
    param(
        [string]$GhPath,
        [string]$RepoName
    )

    Write-Info "Setting VIBESTATS_GH_TOKEN Actions secret..."
    $token = Get-GhAuthToken -GhPath $GhPath
    Set-GhSecretValue -GhPath $GhPath -RepoName $RepoName -SecretName "VIBESTATS_GH_TOKEN" -SecretValue $token -NonFatal | Out-Null
}

function Configure-VibestatsDataRepository {
    param(
        [string]$GhPath,
        [hashtable]$SetupInfo
    )

    $repoName = [string]$SetupInfo["RepoName"]
    $gitHubUser = [string]$SetupInfo["GitHubUser"]

    if (-not $repoName -or -not $gitHubUser) {
        Fail "Setup info did not include RepoName/GitHubUser."
    }

    Write-AggregateWorkflow -GhPath $GhPath -RepoName $repoName
    Set-VibestatsTokenSecret -GhPath $GhPath -RepoName $repoName -GitHubUser $gitHubUser
    Set-GithubContributionsTokenSecret -GhPath $GhPath -RepoName $repoName
}

function Get-GithubFileContent {
    param(
        [string]$GhPath,
        [string]$RepoName,
        [string]$Path
    )

    $responseText = & $GhPath api "repos/$RepoName/contents/$Path" 2>$null
    if ($LASTEXITCODE -ne 0) {
        return $null
    }

    try {
        $response = (($responseText | Out-String).Trim()) | ConvertFrom-Json
        return @{
            Sha = [string]$response.sha
            Content = Decode-GitHubContent -EncodedContent ([string]$response.content)
        }
    }
    catch {
        return $null
    }
}

function Add-ProfileReadmeMarkers {
    param(
        [string]$GhPath,
        [string]$GitHubUser
    )

    if ($SkipReadmeMarkers) {
        Write-Warn "Skipping profile README marker injection."
        return
    }

    $profileRepo = "$GitHubUser/$GitHubUser"
    $readme = Get-GithubFileContent -GhPath $GhPath -RepoName $profileRepo -Path "README.md"

    if (-not $readme) {
        Write-Warn "Could not access $profileRepo/README.md. Add vibestats markers manually if desired."
        Write-Warn "See: https://vibestats.dev/docs/quickstart"
        return
    }

    $content = [string]$readme["Content"]
    if ($content -match "<!-- vibestats-start -->") {
        Write-Success "vibestats README markers already present; skipping."
        return
    }

    $markerBlock = @"
<!-- vibestats-start -->
[![vibestats](https://raw.githubusercontent.com/$GitHubUser/$GitHubUser/main/vibestats/heatmap.svg)](https://vibestats.dev/$GitHubUser)

[View interactive dashboard ->](https://vibestats.dev/$GitHubUser)
<!-- vibestats-end -->
"@

    $updatedContent = $content.TrimEnd() + [Environment]::NewLine + [Environment]::NewLine + $markerBlock + [Environment]::NewLine
    $encoded = Encode-GitHubContent -Content $updatedContent

    $ghArgs = @(
        "api",
        "repos/$profileRepo/contents/README.md",
        "--method",
        "PUT",
        "--field",
        "message=Add vibestats heatmap markers",
        "--field",
        "content=$encoded",
        "--field",
        "sha=$($readme["Sha"])"
    )

    & $GhPath @ghArgs *> $null
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "Could not update $profileRepo/README.md. Add vibestats markers manually if desired."
        return
    }

    Write-Success "vibestats markers added to $profileRepo/README.md"
}

function Run-PostInstallBackfill {
    param([string]$ExePath)

    if ($SkipBackfill) {
        Write-Warn "Skipping post-install backfill."
        return
    }

    Write-Info "Running post-install backfill..."
    & $ExePath sync --backfill
    if ($LASTEXITCODE -ne 0) {
        Write-Warn "Backfill completed with errors. Retry later with: vibestats sync --backfill"
        return
    }

    Write-Success "Backfill complete."
}

function Invoke-InitialAggregateWorkflow {
    param(
        [string]$GhPath,
        [string]$RepoName
    )

    if ($SkipInitialWorkflowDispatch) {
        Write-Warn "Skipping initial aggregate workflow dispatch."
        return
    }

    Write-Info "Triggering initial aggregate workflow run..."

    for ($i = 1; $i -le 5; $i++) {
        & $GhPath api "repos/$RepoName/actions/workflows/aggregate.yml/dispatches" `
            --method POST `
            --field "ref=main" *> $null

        if ($LASTEXITCODE -eq 0) {
            Write-Success "Aggregate workflow triggered. Your heatmap will be ready in a few minutes."
            return
        }

        Write-Info "Workflow not ready yet, retrying in ${i}s..."
        Start-Sleep -Seconds $i
    }

    Write-Warn "Could not trigger aggregate workflow automatically."
    Write-Warn "Run it manually at: https://github.com/$RepoName/actions/workflows/aggregate.yml"
}

function Complete-VibestatsAccountSetup {
    param(
        [string]$GhPath,
        [string]$ExePath,
        [hashtable]$SetupInfo
    )

    $repoName = [string]$SetupInfo["RepoName"]
    $gitHubUser = [string]$SetupInfo["GitHubUser"]

    if (-not $repoName -or -not $gitHubUser) {
        Fail "Setup info did not include RepoName/GitHubUser."
    }

    Add-ProfileReadmeMarkers -GhPath $GhPath -GitHubUser $gitHubUser
    Run-PostInstallBackfill -ExePath $ExePath
    Invoke-InitialAggregateWorkflow -GhPath $GhPath -RepoName $repoName
}

function Get-ReleaseAsset {
    param(
        [string]$RepoName,
        [string]$ReleaseVersion,
        [string]$AssetName
    )

    if ($ReleaseVersion -eq "latest") {
        $releaseUrl = "https://api.github.com/repos/$RepoName/releases/latest"
        Write-Info "Fetching latest release metadata..."
    } else {
        $releaseUrl = "https://api.github.com/repos/$RepoName/releases/tags/$ReleaseVersion"
        Write-Info "Fetching release metadata for $ReleaseVersion..."
    }

    $release = Invoke-RestMethod -UseBasicParsing -Uri $releaseUrl
    $asset = $release.assets | Where-Object { $_.name -eq $AssetName } | Select-Object -First 1

    if (-not $asset) {
        $available = ($release.assets | ForEach-Object { $_.name }) -join [Environment]::NewLine
        Fail "Asset '$AssetName' was not found in release $($release.tag_name). Available assets:$([Environment]::NewLine)$available"
    }

    return @{
        Tag = $release.tag_name
        Url = $asset.browser_download_url
    }
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

function ConvertTo-Hashtable {
    param([AllowNull()]$InputObject)

    if ($null -eq $InputObject) {
        return $null
    }

    if ($InputObject -is [System.Collections.IDictionary]) {
        $hash = @{}
        foreach ($key in $InputObject.Keys) {
            $hash[$key] = ConvertTo-Hashtable $InputObject[$key]
        }
        return $hash
    }

    if ($InputObject -is [System.Management.Automation.PSCustomObject]) {
        $hash = @{}
        foreach ($property in $InputObject.PSObject.Properties) {
            $hash[$property.Name] = ConvertTo-Hashtable $property.Value
        }
        return $hash
    }

    if (($InputObject -is [System.Collections.IEnumerable]) -and -not ($InputObject -is [string])) {
        $items = @()
        foreach ($item in $InputObject) {
            $items += ,(ConvertTo-Hashtable $item)
        }
        return ,$items
    }

    return $InputObject
}

function Read-JsonHashtable {
    param([string]$Path)

    if (-not (Test-Path -LiteralPath $Path)) {
        return @{}
    }

    try {
        $raw = Get-Content -LiteralPath $Path -Raw
        if (-not $raw.Trim()) {
            return @{}
        }

        $parsed = $raw | ConvertFrom-Json
        $converted = ConvertTo-Hashtable $parsed
        if ($converted -is [hashtable]) {
            return $converted
        }

        Write-Warn "Ignoring non-object JSON document: $Path"
        return @{}
    }
    catch {
        Write-Warn "Could not parse $Path. Replacing it with a fresh JSON object."
        return @{}
    }
}

function Write-JsonHashtable {
    param(
        [string]$Path,
        [hashtable]$Data
    )

    $parent = Split-Path -Parent $Path
    if ($parent) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }

    $json = $Data | ConvertTo-Json -Depth 100
    [System.IO.File]::WriteAllText($Path, ($json + [Environment]::NewLine), $Utf8NoBom)
}

function New-VibestatsHookCommand {
    param(
        [string]$ExePath,
        [string]$Arguments
    )

    $quotedExe = '"' + $ExePath.Replace('"', '\"') + '"'
    if ($Arguments.Trim()) {
        return "$quotedExe $Arguments"
    }

    return $quotedExe
}

function Test-LegacyVibestatsHookCommand {
    param([AllowNull()]$Command)

    if (-not ($Command -is [string])) {
        return $false
    }

    $trimmed = $Command.Trim()
    return ($trimmed -eq "vibestats" -or $trimmed.StartsWith("vibestats "))
}

function Ensure-HookCommand {
    param(
        [hashtable]$Hooks,
        [string]$HookName,
        [string]$Command,
        [bool]$Async
    )

    $groups = @()
    if ($Hooks.ContainsKey($HookName) -and $Hooks[$HookName]) {
        $groups = @($Hooks[$HookName])
    }

    $present = $false

    for ($groupIndex = 0; $groupIndex -lt $groups.Count; $groupIndex++) {
        $group = $groups[$groupIndex]
        if (-not ($group -is [hashtable])) {
            continue
        }

        if (-not $group.ContainsKey("hooks") -or -not $group["hooks"]) {
            $group["hooks"] = @()
        }

        $innerHooks = @($group["hooks"])
        for ($hookIndex = 0; $hookIndex -lt $innerHooks.Count; $hookIndex++) {
            $hook = $innerHooks[$hookIndex]
            if (-not ($hook -is [hashtable])) {
                continue
            }

            $oldCommand = $hook["command"]
            if ($oldCommand -eq $Command) {
                $present = $true
            } elseif (Test-LegacyVibestatsHookCommand -Command $oldCommand) {
                $hook["command"] = $Command
                $present = $true
            }
        }

        $group["hooks"] = @($innerHooks)
    }

    if (-not $present) {
        $hookEntry = @{
            type = "command"
            command = $Command
        }

        if ($Async) {
            $hookEntry["async"] = $true
        }

        $groups += @{
            hooks = @($hookEntry)
        }
    }

    $Hooks[$HookName] = @($groups)
}

function Configure-ClaudeHooks {
    param([string]$ExePath)

    $claudeDir = Join-Path $env:USERPROFILE ".claude"
    $settingsPath = Join-Path $claudeDir "settings.json"

    if (-not (Test-Path -LiteralPath $claudeDir)) {
        Write-Warn "Claude Code settings directory not found at $claudeDir; skipping Claude hook configuration."
        Write-Warn "Run Claude Code once, then rerun this installer to configure hooks."
        return
    }

    $settings = Read-JsonHashtable -Path $settingsPath
    if (-not $settings.ContainsKey("hooks") -or -not ($settings["hooks"] -is [hashtable])) {
        $settings["hooks"] = @{}
    }

    $syncCommand = New-VibestatsHookCommand -ExePath $ExePath -Arguments "sync"

    Ensure-HookCommand -Hooks $settings["hooks"] -HookName "Stop" -Command $syncCommand -Async $true
    Ensure-HookCommand -Hooks $settings["hooks"] -HookName "SessionStart" -Command $syncCommand -Async $false

    Write-JsonHashtable -Path $settingsPath -Data $settings
    Write-Success "Claude Code hooks configured: $settingsPath"
}

function Enable-CodexHooksFeature {
    param([string]$ConfigPath)

    $parent = Split-Path -Parent $ConfigPath
    if ($parent) {
        New-Item -ItemType Directory -Force -Path $parent | Out-Null
    }

    $lines = @()
    if (Test-Path -LiteralPath $ConfigPath) {
        $lines = @(Get-Content -LiteralPath $ConfigPath)
    }

    $featuresIndex = -1
    $codexHooksIndex = -1

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $trimmed = $lines[$i].Trim()
        if ($trimmed -eq "[features]") {
            $featuresIndex = $i
        } elseif ($trimmed.StartsWith("codex_hooks")) {
            $codexHooksIndex = $i
        }
    }

    if ($codexHooksIndex -ge 0) {
        $lines[$codexHooksIndex] = "codex_hooks = true"
    } elseif ($featuresIndex -ge 0) {
        $insertAt = $featuresIndex + 1
        while ($insertAt -lt $lines.Count -and -not $lines[$insertAt].TrimStart().StartsWith("[")) {
            $insertAt++
        }

        $before = @()
        $after = @()

        if ($insertAt -gt 0) {
            $before = @($lines[0..($insertAt - 1)])
        }

        if ($insertAt -lt $lines.Count) {
            $after = @($lines[$insertAt..($lines.Count - 1)])
        }

        $lines = @($before + "codex_hooks = true" + $after)
    } else {
        if ($lines.Count -gt 0 -and $lines[-1].Trim()) {
            $lines += ""
        }

        $lines += "[features]"
        $lines += "codex_hooks = true"
    }

    [System.IO.File]::WriteAllText($ConfigPath, (($lines -join [Environment]::NewLine) + [Environment]::NewLine), $Utf8NoBom)
}

function Configure-CodexHooks {
    param([string]$ExePath)

    $codexDir = Join-Path $env:USERPROFILE ".codex"
    if (-not (Test-Path -LiteralPath $codexDir)) {
        Write-Info "Codex directory not found at $codexDir; skipping Codex hook configuration."
        return
    }

    $hooksPath = Join-Path $codexDir "hooks.json"
    $configPath = Join-Path $codexDir "config.toml"
    $hooksDoc = Read-JsonHashtable -Path $hooksPath

    if (-not $hooksDoc.ContainsKey("hooks") -or -not ($hooksDoc["hooks"] -is [hashtable])) {
        $hooksDoc["hooks"] = @{}
    }

    $quietSyncCommand = New-VibestatsHookCommand -ExePath $ExePath -Arguments "sync --quiet"

    Ensure-HookCommand -Hooks $hooksDoc["hooks"] -HookName "Stop" -Command $quietSyncCommand -Async $false
    Ensure-HookCommand -Hooks $hooksDoc["hooks"] -HookName "SessionStart" -Command $quietSyncCommand -Async $false

    Write-JsonHashtable -Path $hooksPath -Data $hooksDoc
    Enable-CodexHooksFeature -ConfigPath $configPath

    Write-Success "Codex hooks configured: $hooksPath"
}

function Configure-LocalHooks {
    param([string]$ExePath)

    if ($SkipHookConfiguration) {
        Write-Warn "Skipping Claude/Codex hook configuration."
        return
    }

    Configure-ClaudeHooks -ExePath $ExePath
    Configure-CodexHooks -ExePath $ExePath
}

function Install-Vibestats {
    param([string]$GhPath)

    if (-not $env:LOCALAPPDATA) {
        Fail "LOCALAPPDATA is not set. This installer needs a normal user profile."
    }

    $target = Get-NativeTarget
    $archiveName = "vibestats-$target.zip"
    $checksumName = "$archiveName.sha256"

    Write-Step "Resolve release assets"
    $archiveAsset = Get-ReleaseAsset -RepoName $Repo -ReleaseVersion $Version -AssetName $archiveName
    $checksumAsset = Get-ReleaseAsset -RepoName $Repo -ReleaseVersion $Version -AssetName $checksumName

    if ($archiveAsset.Tag -ne $checksumAsset.Tag) {
        Fail "Release metadata mismatch between archive and checksum assets."
    }

    Write-Info "Selected release: $($archiveAsset.Tag)"

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("vibestats-install-" + [guid]::NewGuid().ToString("N"))
    $extractDir = Join-Path $tempDir "extract"
    New-Item -ItemType Directory -Force -Path $tempDir, $extractDir | Out-Null

    try {
        $zipPath = Join-Path $tempDir $archiveName
        $checksumPath = Join-Path $tempDir $checksumName

        Write-Step "Download"
        Download-File -Uri $archiveAsset.Url -OutFile $zipPath
        Download-File -Uri $checksumAsset.Url -OutFile $checksumPath

        Write-Step "Verify checksum"
        Assert-Checksum -FilePath $zipPath -ChecksumPath $checksumPath

        Write-Step "Extract"
        Expand-Archive -LiteralPath $zipPath -DestinationPath $extractDir -Force

        $exe = Get-ChildItem -LiteralPath $extractDir -Filter "vibestats.exe" -Recurse | Select-Object -First 1
        if (-not $exe) {
            Fail "Archive did not contain vibestats.exe."
        }

        Write-Step "Install binary"
        New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
        $dest = Join-Path $InstallDir "vibestats.exe"
        Copy-Item -LiteralPath $exe.FullName -Destination $dest -Force
        Unblock-File -LiteralPath $dest -ErrorAction SilentlyContinue

        & $dest --version
        if ($LASTEXITCODE -ne 0) {
            Fail "Installed vibestats.exe, but --version failed."
        }

        Add-UserPathEntry -Directory $InstallDir

        Write-Step "Bootstrap vibestats config"
        $setupInfo = Initialize-VibestatsConfig -GhPath $GhPath

        if (-not $SkipVibestatsAuth) {
            Write-Step "Run vibestats auth"
            & $dest auth
            if ($LASTEXITCODE -ne 0) {
                Write-Warn "vibestats auth exited with code $LASTEXITCODE. You can retry later with: vibestats auth"
            }
        } else {
            Write-Warn "Skipping vibestats auth."
        }

        Write-Step "Configure vibestats-data"
        Configure-VibestatsDataRepository -GhPath $GhPath -SetupInfo $setupInfo

        Write-Step "Configure local hooks"
        Configure-LocalHooks -ExePath $dest

        Write-Step "Complete account setup"
        Complete-VibestatsAccountSetup -GhPath $GhPath -ExePath $dest -SetupInfo $setupInfo

        Write-Success "vibestats installed successfully: $dest"
        Write-Host ""
        Write-Host "Open a new terminal, then run:" -ForegroundColor Cyan
        Write-Host "  vibestats --help"
        Write-Host "  vibestats status"
        Write-Host ""
        Write-Success "Windows installer setup complete."
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
Install-Vibestats -GhPath $ghPath

Write-Step "Complete"
Write-Success "Installation complete."
