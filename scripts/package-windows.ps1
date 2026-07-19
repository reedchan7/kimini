[CmdletBinding()]
param(
    [ValidateSet('native', 'web', 'all')]
    [string]$App = 'all',

    [ValidateSet('x86_64', 'aarch64', 'all')]
    [string]$Arch = 'all',

    [switch]$SkipBuild
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$IsWindowsHost = if (Get-Variable IsWindows -ErrorAction SilentlyContinue) {
    $IsWindows
} else {
    $env:OS -eq 'Windows_NT'
}
if (-not $IsWindowsHost) {
    throw 'package-windows.ps1 must run on Windows with Visual Studio 2022 Build Tools.'
}

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $Root
$Dist = if ($env:DIST) { $env:DIST } else { Join-Path $Root 'dist' }
New-Item -ItemType Directory -Force -Path $Dist | Out-Null

$VersionLine = Get-Content 'Cargo.toml' | Select-String '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $VersionLine) {
    throw 'Could not read the package version from Cargo.toml.'
}
$Version = $VersionLine.Matches[0].Groups[1].Value

foreach ($Command in @('cargo', 'rustup', 'cl.exe', 'link.exe')) {
    if (-not (Get-Command $Command -ErrorAction SilentlyContinue)) {
        throw "$Command was not found. Run this script from a Visual Studio 2022 Developer PowerShell."
    }
}

$Architectures = if ($Arch -eq 'all') { @('x86_64', 'aarch64') } else { @($Arch) }
$Applications = if ($App -eq 'all') { @('native', 'web') } else { @($App) }

foreach ($CurrentArch in $Architectures) {
    $Target = "$CurrentArch-pc-windows-msvc"
    rustup target add $Target
    if ($LASTEXITCODE -ne 0) {
        throw "rustup failed to install target $Target."
    }

    foreach ($CurrentApp in $Applications) {
        if ($CurrentApp -eq 'native') {
            $Binary = 'kimini'
            $Product = 'Kimini'
            $Feature = 'native'
        } else {
            $Binary = 'kimini-web'
            $Product = 'Kimini-Web'
            $Feature = 'legacy-web'
        }

        if (-not $SkipBuild) {
            cargo build --locked --release --target $Target --bin $Binary --no-default-features --features $Feature
            if ($LASTEXITCODE -ne 0) {
                throw "cargo failed to build $Binary for $Target."
            }
        }

        $Executable = Join-Path $Root "target\$Target\release\$Binary.exe"
        if (-not (Test-Path $Executable -PathType Leaf)) {
            throw "Release binary not found: $Executable"
        }

        $ArchiveName = "$Product-$Version-windows-$CurrentArch"
        $Staging = Join-Path ([IO.Path]::GetTempPath()) "kimini-package-$PID"
        if (Test-Path $Staging) {
            Remove-Item -Recurse -Force $Staging
        }
        $Bundle = Join-Path $Staging $ArchiveName
        New-Item -ItemType Directory -Force -Path (Join-Path $Bundle 'bin') | Out-Null
        Copy-Item $Executable (Join-Path $Bundle "bin\$Binary.exe")
        Copy-Item 'LICENSE', 'README.md' $Bundle
        Copy-Item 'docs\brand\exports\app-icon-256.png' (Join-Path $Bundle 'Kimini.png')

        $Archive = Join-Path $Dist "$ArchiveName.zip"
        Compress-Archive -Path $Bundle -DestinationPath $Archive -Force
        $Hash = (Get-FileHash -Algorithm SHA256 $Archive).Hash.ToLowerInvariant()
        "$Hash  $([IO.Path]::GetFileName($Archive))" | Set-Content -NoNewline "$Archive.sha256"
        Remove-Item -Recurse -Force $Staging
        Write-Host "created $Archive"
    }
}
