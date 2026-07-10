#Requires -Version 5.1
<#
.SYNOPSIS
    Pester v5 tests for install.ps1 (the Windows native installer).

    install.ps1 is dot-sourced with -FunctionsOnly so its functions become
    available without running the installer. All file I/O happens under
    TestDrive:; nothing here ever touches the real user PATH, registry, or
    APPDATA.
#>

BeforeDiscovery {
    $script:InstallScript = (Resolve-Path (Join-Path $PSScriptRoot "..\..\install.ps1")).Path
}

BeforeAll {
    $InstallScript = (Resolve-Path (Join-Path $PSScriptRoot "..\..\install.ps1")).Path
    . $InstallScript -FunctionsOnly
}

Describe "Ensure-HookCommand" {

    It "adds a new hook entry with async=true when requested" {
        $hooks = @{}
        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command "vibestats.exe sync" -Async $true

        $hooks.ContainsKey("Stop") | Should -BeTrue
        $hooks["Stop"].Count | Should -Be 1
        $group = $hooks["Stop"][0]
        $group["hooks"].Count | Should -Be 1
        $group["hooks"][0]["command"] | Should -Be "vibestats.exe sync"
        $group["hooks"][0]["type"] | Should -Be "command"
        $group["hooks"][0]["async"] | Should -Be $true
    }

    It "adds a new hook entry without an async key when Async is false" {
        $hooks = @{}
        Ensure-HookCommand -Hooks $hooks -HookName "SessionStart" -Command "vibestats.exe sync" -Async $false

        $hook = $hooks["SessionStart"][0]["hooks"][0]
        $hook.ContainsKey("async") | Should -BeFalse
    }

    It "is idempotent -- calling twice with the same command yields exactly one entry" {
        $hooks = @{}
        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command "vibestats.exe sync" -Async $true
        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command "vibestats.exe sync" -Async $true

        $hooks["Stop"].Count | Should -Be 1
        $hooks["Stop"][0]["hooks"].Count | Should -Be 1
    }

    It "updates a legacy 'vibestats ...' command in place instead of adding a duplicate" {
        $hooks = @{
            Stop = @(
                @{
                    hooks = @(
                        @{ type = "command"; command = "vibestats sync" }
                    )
                }
            )
        }

        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command '"C:\Users\me\vibestats.exe" sync' -Async $true

        $hooks["Stop"].Count | Should -Be 1
        $hooks["Stop"][0]["hooks"].Count | Should -Be 1
        $hooks["Stop"][0]["hooks"][0]["command"] | Should -Be '"C:\Users\me\vibestats.exe" sync'
    }

    It "preserves unrelated hook keys already present in the hooks table" {
        $hooks = @{
            PreToolUse = @(
                @{
                    hooks = @(
                        @{ type = "command"; command = "echo pre-existing-hook" }
                    )
                }
            )
        }

        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command "vibestats.exe sync" -Async $true

        $hooks.ContainsKey("PreToolUse") | Should -BeTrue
        $hooks["PreToolUse"][0]["hooks"][0]["command"] | Should -Be "echo pre-existing-hook"
        $hooks.ContainsKey("Stop") | Should -BeTrue
    }

    It "does not duplicate when a matching command already exists alongside other groups" {
        $hooks = @{
            Stop = @(
                @{ hooks = @(@{ type = "command"; command = "echo unrelated" }) },
                @{ hooks = @(@{ type = "command"; command = "vibestats.exe sync" }) }
            )
        }

        Ensure-HookCommand -Hooks $hooks -HookName "Stop" -Command "vibestats.exe sync" -Async $true

        $hooks["Stop"].Count | Should -Be 2
    }
}

Describe "Enable-CodexHooksFeature" {

    It "updates an existing codex_hooks key in place" {
        $path = Join-Path $TestDrive "config1.toml"
        Set-Content -LiteralPath $path -Value @(
            "[features]"
            "codex_hooks = false"
        ) -Encoding UTF8

        Enable-CodexHooksFeature -ConfigPath $path

        $lines = @(Get-Content -LiteralPath $path)
        @($lines | Where-Object { $_.Trim() -eq "codex_hooks = true" }).Count | Should -Be 1
        @($lines | Where-Object { $_ -match "codex_hooks" }).Count | Should -Be 1
    }

    It "inserts codex_hooks into an existing [features] section before the next section header" {
        $path = Join-Path $TestDrive "config2.toml"
        Set-Content -LiteralPath $path -Value @(
            "[features]"
            "some_other_flag = true"
            ""
            "[other]"
            "x = 1"
        ) -Encoding UTF8

        Enable-CodexHooksFeature -ConfigPath $path

        $lines = @(Get-Content -LiteralPath $path)
        $lines | Should -Be @(
            "[features]"
            "some_other_flag = true"
            ""
            "codex_hooks = true"
            "[other]"
            "x = 1"
        )
    }

    It "appends a new [features] section when none exists in a non-empty file" {
        $path = Join-Path $TestDrive "config3.toml"
        Set-Content -LiteralPath $path -Value @(
            'profile = "default"'
        ) -Encoding UTF8

        Enable-CodexHooksFeature -ConfigPath $path

        $lines = @(Get-Content -LiteralPath $path)
        $lines | Should -Be @(
            'profile = "default"'
            ""
            "[features]"
            "codex_hooks = true"
        )
    }

    It "creates the file with a [features] section when the config file does not exist" {
        $path = Join-Path $TestDrive "config4.toml"

        Enable-CodexHooksFeature -ConfigPath $path

        Test-Path -LiteralPath $path | Should -BeTrue
        $lines = @(Get-Content -LiteralPath $path)
        $lines | Should -Be @(
            "[features]"
            "codex_hooks = true"
        )
    }

    It "writes the file as UTF8 without a byte-order mark" {
        $path = Join-Path $TestDrive "config5.toml"

        Enable-CodexHooksFeature -ConfigPath $path

        $bytes = [System.IO.File]::ReadAllBytes($path)
        ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) | Should -BeFalse
    }
}

Describe "Add-UserPathEntry" {
    BeforeEach {
        $script:originalProcessPath = $env:Path
    }

    AfterEach {
        $env:Path = $script:originalProcessPath
    }

    It "adds the directory to the (mocked) user PATH when not already present" {
        Mock Get-UserPathVariable { return "C:\Windows;C:\Windows\System32" }
        Mock Set-UserPathVariable { }

        Add-UserPathEntry -Directory "C:\tools\vibestats"

        Should -Invoke Set-UserPathVariable -Times 1 -ParameterFilter {
            $Value -like "*C:\tools\vibestats*"
        }
    }

    It "does not call Set-UserPathVariable when the directory is already present (case-insensitive)" {
        Mock Get-UserPathVariable { return "C:\Windows;c:\TOOLS\VIBESTATS" }
        Mock Set-UserPathVariable { }

        Add-UserPathEntry -Directory "C:\tools\vibestats"

        Should -Invoke Set-UserPathVariable -Times 0
    }

    It "treats a trailing slash / GetFullPath-normalized entry as already present" {
        Mock Get-UserPathVariable { return "C:\tools\vibestats\" }
        Mock Set-UserPathVariable { }

        Add-UserPathEntry -Directory "C:\tools\vibestats"

        Should -Invoke Set-UserPathVariable -Times 0
    }

    It "is idempotent across repeated calls against the same (mocked) backing store" {
        $store = "C:\Windows"
        Mock Get-UserPathVariable { return $store }
        Mock Set-UserPathVariable { param($Value) $script:store = $Value } -ParameterFilter { $true }

        Add-UserPathEntry -Directory "C:\tools\vibestats"
        Should -Invoke Set-UserPathVariable -Times 1
    }

    It "adds the resolved directory to the current process PATH (Process scope only)" {
        Mock Get-UserPathVariable { return "" }
        Mock Set-UserPathVariable { }

        $env:Path = "C:\Windows"
        $dir = Join-Path $TestDrive "bin"
        New-Item -ItemType Directory -Force -Path $dir | Out-Null

        Add-UserPathEntry -Directory $dir

        $resolved = [System.IO.Path]::GetFullPath($dir).TrimEnd('\')
        ($env:Path -split ';') | Should -Contain $resolved
    }

    It "does not duplicate an entry already present in the process PATH" {
        Mock Get-UserPathVariable { return "" }
        Mock Set-UserPathVariable { }

        $dir = Join-Path $TestDrive "bin2"
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $resolved = [System.IO.Path]::GetFullPath($dir).TrimEnd('\')
        $env:Path = "$resolved;C:\Windows"

        Add-UserPathEntry -Directory $dir

        $count = @($env:Path -split ';' | Where-Object { $_ -eq $resolved }).Count
        $count | Should -Be 1
    }

    It "never invokes Get-UserPathVariable/Set-UserPathVariable when -SkipPathUpdate is set" {
        . $InstallScript -FunctionsOnly -SkipPathUpdate

        Mock Get-UserPathVariable { return "" }
        Mock Set-UserPathVariable { }

        Add-UserPathEntry -Directory "C:\tools\vibestats"

        Should -Invoke Get-UserPathVariable -Times 0
        Should -Invoke Set-UserPathVariable -Times 0

        # Restore functions without -SkipPathUpdate for subsequent tests in this run.
        . $InstallScript -FunctionsOnly
    }
}

Describe "Test-LegacyVibestatsHookCommand" {
    It "returns false for null" {
        Test-LegacyVibestatsHookCommand -Command $null | Should -BeFalse
    }

    It "returns false for a non-string value" {
        Test-LegacyVibestatsHookCommand -Command 123 | Should -BeFalse
    }

    It "returns true for the bare 'vibestats' command" {
        Test-LegacyVibestatsHookCommand -Command "vibestats" | Should -BeTrue
    }

    It "returns true for 'vibestats sync'" {
        Test-LegacyVibestatsHookCommand -Command "vibestats sync" | Should -BeTrue
    }

    It "trims whitespace before matching" {
        Test-LegacyVibestatsHookCommand -Command "  vibestats sync  " | Should -BeTrue
    }

    It "returns false for a command that merely starts with 'vibestats' without a space" {
        Test-LegacyVibestatsHookCommand -Command "vibestats-sync" | Should -BeFalse
    }

    It "returns false for a quoted exe path command" {
        Test-LegacyVibestatsHookCommand -Command '"C:\tools\vibestats.exe" sync' | Should -BeFalse
    }
}

Describe "New-VibestatsHookCommand" {
    It "quotes the exe path and appends args" {
        New-VibestatsHookCommand -ExePath "C:\tools\vibestats.exe" -Arguments "sync" |
            Should -Be '"C:\tools\vibestats.exe" sync'
    }

    It "returns just the quoted exe path when Arguments is empty" {
        New-VibestatsHookCommand -ExePath "C:\tools\vibestats.exe" -Arguments "" |
            Should -Be '"C:\tools\vibestats.exe"'
    }

    It "returns just the quoted exe path when Arguments is whitespace-only" {
        New-VibestatsHookCommand -ExePath "C:\tools\vibestats.exe" -Arguments "   " |
            Should -Be '"C:\tools\vibestats.exe"'
    }

    It "escapes embedded double quotes in the exe path" {
        New-VibestatsHookCommand -ExePath 'C:\tools\vibe"stats.exe' -Arguments "sync" |
            Should -Be '"C:\tools\vibe\"stats.exe" sync'
    }
}

Describe "ConvertTo-Slug" {
    It "lowercases and normalizes punctuation/spaces to single hyphens" {
        ConvertTo-Slug -Value "My Cool PC!!" | Should -Be "my-cool-pc"
    }

    It "truncates to 20 characters and trims a trailing hyphen" {
        $long = "a" * 25
        $slug = ConvertTo-Slug -Value $long
        $slug.Length | Should -Be 20
        $slug | Should -Be ("a" * 20)
    }

    It "falls back to 'machine' for empty input" {
        ConvertTo-Slug -Value "" | Should -Be "machine"
    }

    It "falls back to 'machine' when input has no alphanumeric characters" {
        ConvertTo-Slug -Value "!!!___###" | Should -Be "machine"
    }
}

Describe "ConvertTo-Hashtable" {
    It "returns null for null input" {
        ConvertTo-Hashtable -InputObject $null | Should -BeNullOrEmpty
    }

    It "converts a PSCustomObject (including nested objects) to nested hashtables" {
        $obj = [PSCustomObject]@{
            name  = "test"
            inner = [PSCustomObject]@{ a = 1; b = 2 }
        }

        $result = ConvertTo-Hashtable -InputObject $obj
        $result | Should -BeOfType [hashtable]
        $result["name"] | Should -Be "test"
        $result["inner"] | Should -BeOfType [hashtable]
        $result["inner"]["a"] | Should -Be 1
    }

    It "preserves arrays as arrays" {
        $obj = [PSCustomObject]@{
            items = @([PSCustomObject]@{ x = 1 }, [PSCustomObject]@{ x = 2 })
        }

        $result = ConvertTo-Hashtable -InputObject $obj
        , $result["items"] | Should -BeOfType [array]
        $result["items"].Count | Should -Be 2
        $result["items"][0]["x"] | Should -Be 1
    }

    It "passes scalars through unchanged" {
        ConvertTo-Hashtable -InputObject 42 | Should -Be 42
        ConvertTo-Hashtable -InputObject "hello" | Should -Be "hello"
    }
}

Describe "Read-JsonHashtable / Write-JsonHashtable" {
    It "returns an empty hashtable when the file does not exist" {
        $path = Join-Path $TestDrive "missing.json"
        $result = Read-JsonHashtable -Path $path
        $result | Should -BeOfType [hashtable]
        $result.Count | Should -Be 0
    }

    It "returns an empty hashtable when the file is empty" {
        $path = Join-Path $TestDrive "empty.json"
        Set-Content -LiteralPath $path -Value "" -NoNewline
        $result = Read-JsonHashtable -Path $path
        $result | Should -BeOfType [hashtable]
        $result.Count | Should -Be 0
    }

    It "returns an empty hashtable for malformed JSON" {
        $path = Join-Path $TestDrive "malformed.json"
        Set-Content -LiteralPath $path -Value "{ this is not json "
        $result = Read-JsonHashtable -Path $path
        $result | Should -BeOfType [hashtable]
        $result.Count | Should -Be 0
    }

    It "returns an empty hashtable for non-object JSON (array)" {
        $path = Join-Path $TestDrive "array.json"
        Set-Content -LiteralPath $path -Value "[1, 2, 3]"
        $result = Read-JsonHashtable -Path $path
        $result | Should -BeOfType [hashtable]
        $result.Count | Should -Be 0
    }

    It "returns an empty hashtable for non-object JSON (scalar)" {
        $path = Join-Path $TestDrive "scalar.json"
        Set-Content -LiteralPath $path -Value "42"
        $result = Read-JsonHashtable -Path $path
        $result | Should -BeOfType [hashtable]
        $result.Count | Should -Be 0
    }

    It "round-trips a hashtable through Write-JsonHashtable / Read-JsonHashtable" {
        $path = Join-Path $TestDrive "roundtrip.json"
        $data = @{ hooks = @{ Stop = @() }; machine_id = "abc-123" }

        Write-JsonHashtable -Path $path -Data $data
        $result = Read-JsonHashtable -Path $path

        $result["machine_id"] | Should -Be "abc-123"
        $result["hooks"] | Should -BeOfType [hashtable]
    }

    It "writes UTF8 without a byte-order mark" {
        $path = Join-Path $TestDrive "utf8.json"
        Write-JsonHashtable -Path $path -Data @{ a = 1 }

        $bytes = [System.IO.File]::ReadAllBytes($path)
        ($bytes.Length -ge 3 -and $bytes[0] -eq 0xEF -and $bytes[1] -eq 0xBB -and $bytes[2] -eq 0xBF) | Should -BeFalse
    }
}

Describe "Assert-Checksum" {
    BeforeEach {
        $script:filePath = Join-Path $TestDrive "artifact.zip"
        Set-Content -LiteralPath $script:filePath -Value "vibestats test payload" -NoNewline
        $script:actualHash = (Get-FileHash -LiteralPath $script:filePath -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    It "passes when the checksum file contains the matching SHA256 hash" {
        $checksumPath = Join-Path $TestDrive "artifact.zip.sha256"
        Set-Content -LiteralPath $checksumPath -Value "$($script:actualHash)  artifact.zip"

        { Assert-Checksum -FilePath $script:filePath -ChecksumPath $checksumPath } | Should -Not -Throw
    }

    It "fails when the checksum does not match" {
        Mock Fail { throw "FAIL: $Message" }
        $checksumPath = Join-Path $TestDrive "artifact.zip.sha256"
        $wrongHash = "0" * 64
        Set-Content -LiteralPath $checksumPath -Value "$wrongHash  artifact.zip"

        { Assert-Checksum -FilePath $script:filePath -ChecksumPath $checksumPath } | Should -Throw
        Should -Invoke Fail -Times 1
    }

    It "fails when the checksum file contains no parseable SHA256 hash" {
        Mock Fail { throw "FAIL: $Message" }
        $checksumPath = Join-Path $TestDrive "artifact.zip.sha256"
        Set-Content -LiteralPath $checksumPath -Value "not a hash"

        { Assert-Checksum -FilePath $script:filePath -ChecksumPath $checksumPath } | Should -Throw
        Should -Invoke Fail -Times 1
    }
}

Describe "Get-NativeTarget" {
    BeforeEach {
        $script:origArch = $env:PROCESSOR_ARCHITECTURE
        $script:origArchw6432 = $env:PROCESSOR_ARCHITEW6432
    }

    AfterEach {
        $env:PROCESSOR_ARCHITECTURE = $script:origArch
        $env:PROCESSOR_ARCHITEW6432 = $script:origArchw6432
    }

    It "returns the x64 target for AMD64" {
        $env:PROCESSOR_ARCHITECTURE = "AMD64"
        $env:PROCESSOR_ARCHITEW6432 = $null

        Get-NativeTarget | Should -Be "x86_64-pc-windows-msvc"
    }

    It "returns the x64 target for x86_64" {
        $env:PROCESSOR_ARCHITECTURE = "x86_64"
        $env:PROCESSOR_ARCHITEW6432 = $null

        Get-NativeTarget | Should -Be "x86_64-pc-windows-msvc"
    }

    It "resolves WOW64 (x86 process on AMD64 host) via PROCESSOR_ARCHITEW6432" {
        $env:PROCESSOR_ARCHITECTURE = "x86"
        $env:PROCESSOR_ARCHITEW6432 = "AMD64"

        Get-NativeTarget | Should -Be "x86_64-pc-windows-msvc"
    }

    It "fails for an unsupported architecture (ARM64)" {
        Mock Fail { throw "FAIL: $Message" }
        $env:PROCESSOR_ARCHITECTURE = "ARM64"
        $env:PROCESSOR_ARCHITEW6432 = $null

        { Get-NativeTarget } | Should -Throw
        Should -Invoke Fail -Times 1
    }

    It "fails for plain x86 with no WOW64 override" {
        Mock Fail { throw "FAIL: $Message" }
        $env:PROCESSOR_ARCHITECTURE = "x86"
        $env:PROCESSOR_ARCHITEW6432 = $null

        { Get-NativeTarget } | Should -Throw
        Should -Invoke Fail -Times 1
    }
}

Describe "Assert-GhVersion" {
    BeforeAll {
        function New-FakeGh {
            param([string]$Name, [string]$Output, [int]$ExitCode = 0)

            $path = Join-Path $TestDrive $Name
            $body = @(
                "@echo off"
                "echo $Output"
                "exit /b $ExitCode"
            ) -join [Environment]::NewLine
            Set-Content -LiteralPath $path -Value $body -Encoding ASCII
            return $path
        }
    }

    It "accepts a modern gh version (>= 2.0)" {
        $gh = New-FakeGh -Name "gh-modern.cmd" -Output "gh version 2.40.0 (2023-01-01)"
        { Assert-GhVersion -GhPath $gh } | Should -Not -Throw
    }

    It "rejects a gh version older than 2.0" {
        Mock Fail { throw "FAIL: $Message" }
        $gh = New-FakeGh -Name "gh-old.cmd" -Output "gh version 1.14.0 (2021-01-01)"

        { Assert-GhVersion -GhPath $gh } | Should -Throw
        Should -Invoke Fail -Times 1
    }

    It "warns (does not fail) when the version string cannot be parsed" {
        $gh = New-FakeGh -Name "gh-unparseable.cmd" -Output "not a version string"
        { Assert-GhVersion -GhPath $gh } | Should -Not -Throw
    }

    It "fails when gh --version itself exits non-zero" {
        Mock Fail { throw "FAIL: $Message" }
        $gh = New-FakeGh -Name "gh-broken.cmd" -Output "boom" -ExitCode 1

        { Assert-GhVersion -GhPath $gh } | Should -Throw
        Should -Invoke Fail -Times 1
    }
}

Describe "gh-network registry functions" {
    # These functions invoke gh inline as `& $GhPath <args>`, which Pester's
    # Mock cannot intercept. Instead we point -GhPath at a small stub .ps1
    # script that logs every argument list it receives (so assertions can find
    # the PUT call regardless of how many GET calls precede it) and returns a
    # canned response/exit code configured per test via Set-GhResponse.

    BeforeAll {
        function New-GhStub {
            param([string]$Dir)

            $stubPath = Join-Path $Dir "gh-stub.ps1"
            $logPath = Join-Path $Dir "gh-log.txt"
            $getResponsePath = Join-Path $Dir "gh-get-response.txt"
            $getExitPath = Join-Path $Dir "gh-get-exit.txt"
            $putResponsePath = Join-Path $Dir "gh-put-response.txt"
            $putExitPath = Join-Path $Dir "gh-put-exit.txt"

            $content = @"
param()

`$logPath = '$logPath'
Add-Content -LiteralPath `$logPath -Value '###CALL###'
foreach (`$a in `$args) {
    Add-Content -LiteralPath `$logPath -Value `$a
}

`$isPut = `$false
for (`$i = 0; `$i -lt `$args.Count; `$i++) {
    if (`$args[`$i] -eq '--method' -and (`$i + 1) -lt `$args.Count -and `$args[`$i + 1] -eq 'PUT') {
        `$isPut = `$true
    }
}

if (`$isPut) {
    `$respPath = '$putResponsePath'
    `$exitPath = '$putExitPath'
} else {
    `$respPath = '$getResponsePath'
    `$exitPath = '$getExitPath'
}

if (Test-Path -LiteralPath `$respPath) {
    Get-Content -LiteralPath `$respPath -Raw
}

`$code = 0
if (Test-Path -LiteralPath `$exitPath) {
    `$code = [int](Get-Content -LiteralPath `$exitPath -Raw).Trim()
}

exit `$code
"@

            Set-Content -LiteralPath $stubPath -Value $content -Encoding UTF8
            return $stubPath
        }

        function Set-GhResponse {
            param(
                [string]$Dir,
                [ValidateSet("Get", "Put")]
                [string]$Kind,
                [string]$Content = "",
                [int]$ExitCode = 0
            )

            $suffix = $Kind.ToLowerInvariant()
            Set-Content -LiteralPath (Join-Path $Dir "gh-$suffix-response.txt") -Value $Content -NoNewline
            Set-Content -LiteralPath (Join-Path $Dir "gh-$suffix-exit.txt") -Value "$ExitCode" -NoNewline
        }

        function Get-GhStubCalls {
            param([string]$Dir)

            $logPath = Join-Path $Dir "gh-log.txt"
            if (-not (Test-Path -LiteralPath $logPath)) {
                return @()
            }

            $lines = @(Get-Content -LiteralPath $logPath)
            $calls = @()
            $current = $null

            foreach ($line in $lines) {
                if ($line -eq "###CALL###") {
                    if ($null -ne $current) {
                        $calls += ,(@($current))
                    }
                    $current = @()
                } elseif ($null -ne $current) {
                    $current += $line
                }
            }

            if ($null -ne $current) {
                $calls += ,(@($current))
            }

            return ,$calls
        }

        function Find-GhPutCall {
            param([string]$Dir)

            $calls = Get-GhStubCalls -Dir $Dir
            foreach ($call in $calls) {
                for ($i = 0; $i -lt $call.Count; $i++) {
                    if ($call[$i] -eq "--method" -and ($i + 1) -lt $call.Count -and $call[$i + 1] -eq "PUT") {
                        return $call
                    }
                }
            }

            return $null
        }

        function Get-GhFieldValue {
            param(
                [array]$Call,
                [string]$FieldName
            )

            if ($null -eq $Call) {
                return $null
            }

            for ($i = 0; $i -lt $Call.Count; $i++) {
                if ($Call[$i] -eq "--field" -and ($i + 1) -lt $Call.Count) {
                    $fieldValue = $Call[$i + 1]
                    if ($fieldValue.StartsWith("$FieldName=")) {
                        return $fieldValue.Substring($FieldName.Length + 1)
                    }
                }
            }

            return $null
        }
    }

    BeforeEach {
        $script:ghDir = Join-Path $TestDrive ("gh-" + [guid]::NewGuid().ToString("N"))
        New-Item -ItemType Directory -Force -Path $script:ghDir | Out-Null
        $script:ghStub = New-GhStub -Dir $script:ghDir
    }

    Context "Get-RegistryDocument" {
        It "returns a fresh empty registry when gh exits non-zero (404)" {
            Set-GhResponse -Dir $script:ghDir -Kind Get -ExitCode 1

            $result = Get-RegistryDocument -GhPath $script:ghStub -RepoName "someuser/vibestats-data"

            $result["machines"].Count | Should -Be 0
        }

        It "decodes a contents-API envelope into a registry document" {
            $registryJson = @{
                machines = @(
                    @{ machine_id = "m1"; hostname = "h1"; status = "active"; last_seen = "2024-01-01T00:00:00Z" }
                )
            } | ConvertTo-Json -Depth 10
            $encoded = Encode-GitHubContent -Content $registryJson
            $envelope = @{ content = $encoded; sha = "deadbeef" } | ConvertTo-Json

            Set-GhResponse -Dir $script:ghDir -Kind Get -Content $envelope -ExitCode 0

            $result = Get-RegistryDocument -GhPath $script:ghStub -RepoName "someuser/vibestats-data"

            $result["machines"].Count | Should -Be 1
            $result["machines"][0]["machine_id"] | Should -Be "m1"
        }

        It "falls back to a fresh registry when the decoded content is malformed JSON" {
            $encoded = Encode-GitHubContent -Content "{ not valid json"
            $envelope = @{ content = $encoded; sha = "xyz" } | ConvertTo-Json

            Set-GhResponse -Dir $script:ghDir -Kind Get -Content $envelope -ExitCode 0

            $result = Get-RegistryDocument -GhPath $script:ghStub -RepoName "someuser/vibestats-data"

            $result["machines"].Count | Should -Be 0
        }
    }

    Context "Save-RegistryDocument" {
        # Save-RegistryDocument delegates to Write-GithubFile, which issues a
        # GET first to look up the sha before the PUT.
        It "PUTs registry.json with no sha field when the file does not exist remotely" {
            Set-GhResponse -Dir $script:ghDir -Kind Get -ExitCode 1
            Set-GhResponse -Dir $script:ghDir -Kind Put -ExitCode 0

            $doc = @{
                machines = @(
                    @{ machine_id = "new-machine"; hostname = "h"; status = "active"; last_seen = "2024-01-01T00:00:00Z" }
                )
            }

            Save-RegistryDocument -GhPath $script:ghStub -RepoName "someuser/vibestats-data" -Document $doc -Message "chore: register machine new-machine"

            $putCall = Find-GhPutCall -Dir $script:ghDir
            $putCall | Should -Not -BeNullOrEmpty

            $encodedContent = Get-GhFieldValue -Call $putCall -FieldName "content"
            $decoded = Decode-GitHubContent -EncodedContent $encodedContent | ConvertFrom-Json
            $decoded.machines[0].machine_id | Should -Be "new-machine"

            Get-GhFieldValue -Call $putCall -FieldName "sha" | Should -BeNullOrEmpty
        }

        It "PUTs registry.json carrying the sha when the file exists remotely" {
            # Real contents-API file responses always include a content field.
            $existingShaResponse = @{
                sha = "abc123sha"
                content = (Encode-GitHubContent -Content '{"machines":[]}')
            } | ConvertTo-Json
            Set-GhResponse -Dir $script:ghDir -Kind Get -Content $existingShaResponse -ExitCode 0
            Set-GhResponse -Dir $script:ghDir -Kind Put -ExitCode 0

            $doc = @{ machines = @() }

            Save-RegistryDocument -GhPath $script:ghStub -RepoName "someuser/vibestats-data" -Document $doc -Message "chore: register machine m"

            $putCall = Find-GhPutCall -Dir $script:ghDir
            Get-GhFieldValue -Call $putCall -FieldName "sha" | Should -Be "abc123sha"
        }
    }

    Context "Write-GithubFile" {
        It "uses AddMessage and omits sha when the file does not exist remotely" {
            Set-GhResponse -Dir $script:ghDir -Kind Get -ExitCode 1
            Set-GhResponse -Dir $script:ghDir -Kind Put -ExitCode 0

            Write-GithubFile -GhPath $script:ghStub -RepoName "someuser/repo" -Path "foo.txt" -Content "hello" -AddMessage "ADD" -UpdateMessage "UPDATE"

            $putCall = Find-GhPutCall -Dir $script:ghDir
            $putCall | Should -Not -BeNullOrEmpty
            Get-GhFieldValue -Call $putCall -FieldName "message" | Should -Be "ADD"
            Get-GhFieldValue -Call $putCall -FieldName "sha" | Should -BeNullOrEmpty
        }

        It "uses UpdateMessage and carries the sha when the file already exists remotely" {
            # Real contents-API file responses always include a content field.
            $existingShaResponse = @{
                sha = "existingsha123"
                content = (Encode-GitHubContent -Content "old contents")
            } | ConvertTo-Json
            Set-GhResponse -Dir $script:ghDir -Kind Get -Content $existingShaResponse -ExitCode 0
            Set-GhResponse -Dir $script:ghDir -Kind Put -ExitCode 0

            Write-GithubFile -GhPath $script:ghStub -RepoName "someuser/repo" -Path "foo.txt" -Content "hello" -AddMessage "ADD" -UpdateMessage "UPDATE"

            $putCall = Find-GhPutCall -Dir $script:ghDir
            Get-GhFieldValue -Call $putCall -FieldName "message" | Should -Be "UPDATE"
            Get-GhFieldValue -Call $putCall -FieldName "sha" | Should -Be "existingsha123"
        }
    }

    Context "Register-VibestatsMachine" {
        # Get-RegistryDocument/Save-RegistryDocument are named functions, so
        # Pester Mock intercepts them directly -- no gh stub needed here.
        It "appends a new machine entry when no matching machine_id exists" {
            Mock Get-RegistryDocument { return @{ machines = @() } }
            Mock Save-RegistryDocument { }

            Register-VibestatsMachine -GhPath "unused" -RepoName "someuser/vibestats-data" -MachineId "new-machine-abc123"

            Should -Invoke Save-RegistryDocument -Times 1 -ParameterFilter {
                $Document["machines"].Count -eq 1 -and $Document["machines"][0]["machine_id"] -eq "new-machine-abc123"
            }
        }

        It "updates an existing machine_id in place and preserves other machines" {
            Mock Get-RegistryDocument {
                return @{
                    machines = @(
                        @{ machine_id = "new-machine-abc123"; hostname = "oldhost"; status = "inactive"; last_seen = "2020-01-01T00:00:00Z" }
                        @{ machine_id = "other-machine"; hostname = "otherhost"; status = "active"; last_seen = "2021-01-01T00:00:00Z" }
                    )
                }
            }
            Mock Save-RegistryDocument { }

            Register-VibestatsMachine -GhPath "unused" -RepoName "someuser/vibestats-data" -MachineId "new-machine-abc123"

            Should -Invoke Save-RegistryDocument -Times 1 -ParameterFilter {
                $machines = @($Document["machines"])
                $other = $machines | Where-Object { $_["machine_id"] -eq "other-machine" }
                $updated = $machines | Where-Object { $_["machine_id"] -eq "new-machine-abc123" }
                $machines.Count -eq 2 -and $null -ne $other -and $updated["status"] -eq "active" -and $updated["hostname"] -eq [System.Net.Dns]::GetHostName()
            }
        }
    }
}
