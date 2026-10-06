param(
    [ValidateSet('release')]
    [string]$Profile = 'release',
    [string]$IsccPath = '',
    [string]$RcPath = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ([Environment]::OSVersion.Platform -ne [PlatformID]::Win32NT) {
    throw 'Build the Windows installer on Windows with the MSVC Rust toolchain.'
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$target = 'x86_64-pc-windows-msvc'

foreach ($requiredFile in @(
    'Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md',
    'assets\icons\key-jolt.ico', 'assets\icons\key-jolt.png', 'assets\icons\key-jolt.svg',
    'packaging\windows\resource.ps1', 'packaging\windows\KeyJolt.iss'
)) {
    if (-not (Test-Path -LiteralPath (Join-Path $repoRoot $requiredFile) -PathType Leaf)) {
        throw "Missing $requiredFile. Use the complete repository with its original lockfile and license."
    }
}

if ([string]::IsNullOrWhiteSpace($IsccPath)) {
    $compilerCommand = Get-Command 'ISCC.exe' -CommandType Application -ErrorAction SilentlyContinue
    if ($null -ne $compilerCommand) {
        $IsccPath = $compilerCommand.Source
    } else {
        $programDirectories = @(
            $env:ProgramFiles,
            ${env:ProgramFiles(x86)},
            (Join-Path $env:LOCALAPPDATA 'Programs')
        ) | Select-Object -Unique
        foreach ($directory in $programDirectories) {
            if ([string]::IsNullOrWhiteSpace($directory) -or
                -not (Test-Path -LiteralPath $directory -PathType Container)) {
                continue
            }
            $installations = Get-ChildItem -LiteralPath $directory -Directory -Filter 'Inno Setup *' -ErrorAction SilentlyContinue |
                Sort-Object Name -Descending
            foreach ($installation in $installations) {
                $candidate = Join-Path $installation.FullName 'ISCC.exe'
                if (Test-Path -LiteralPath $candidate -PathType Leaf) {
                    $IsccPath = $candidate
                    break
                }
            }
            if (-not [string]::IsNullOrWhiteSpace($IsccPath)) {
                break
            }
        }
    }
}

if ([string]::IsNullOrWhiteSpace($IsccPath) -or -not (Test-Path -LiteralPath $IsccPath -PathType Leaf)) {
    throw 'Could not find ISCC.exe. Install Inno Setup 6.3 or newer, or pass -IsccPath with the actual path to ISCC.exe.'
}

Push-Location $repoRoot
try {
    $metadataJson = & cargo metadata --locked --no-deps --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw 'Cargo metadata failed. Keep Cargo.lock consistent with Cargo.toml.'
    }
    $metadata = ($metadataJson -join "`n") | ConvertFrom-Json
    $package = $metadata.packages | Where-Object { $_.name -eq 'key-jolt' } | Select-Object -First 1
    if ($null -eq $package) {
        throw 'The key-jolt package was not found in Cargo metadata.'
    }
    $version = [string]$package.version
    if ($version -notmatch '^\d+\.\d+\.\d+$') {
        throw 'This installer recipe requires a stable three-part version, such as 0.1.0.'
    }
    if ($package.authors.Count -eq 0) {
        throw 'Set package.authors in Cargo.toml before building the installer.'
    }
    $publisher = [string]$package.authors[0]
    if ([string]::IsNullOrWhiteSpace($publisher) -or $publisher -match '["\x00-\x1f\x7f]') {
        throw 'The first Cargo author must be nonempty and contain no quotes or control characters.'
    }
    $description = [string]$package.description
    if ([string]::IsNullOrWhiteSpace($description)) {
        $description = 'KeyJolt'
    }
    $resourceArguments = @{
        Version = $version
        Publisher = $publisher
        Description = $description
        TargetDirectory = [string]$metadata.target_directory
        IconPath = (Join-Path $repoRoot 'assets\icons\key-jolt.ico')
        RcPath = $RcPath
    }
    $resourcePath = & (Join-Path $PSScriptRoot 'resource.ps1') @resourceArguments

    $buildArguments = @(
        'rustc', '--locked', '--profile', $Profile, '--target', $target,
        '--bin', 'key-jolt', '--', '-C', "link-arg=$resourcePath"
    )
    & cargo @buildArguments
    if ($LASTEXITCODE -ne 0) {
        throw 'The Windows release build failed.'
    }
    $binaryPath = Join-Path $metadata.target_directory "$target\$Profile\key-jolt.exe"
    if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
        throw "The release executable was not found at $binaryPath."
    }

    $outputDirectory = Join-Path $repoRoot "dist\windows\$version"
    New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
    $compilerArguments = @(
        "/DAppVersion=$version",
        "/DAppPublisher=$publisher",
        "/DAppBinary=$binaryPath",
        "/O$outputDirectory",
        (Join-Path $PSScriptRoot 'KeyJolt.iss')
    )
    & $IsccPath @compilerArguments
    if ($LASTEXITCODE -ne 0) {
        throw 'Inno Setup compilation failed.'
    }

    $installerName = "KeyJolt-$version-windows-x64-Setup.exe"
    $installerPath = Join-Path $outputDirectory $installerName
    if (-not (Test-Path -LiteralPath $installerPath -PathType Leaf)) {
        throw 'Inno Setup did not produce the expected installer.'
    }
    Write-Host "Installer created: $installerPath"
    $portableName = "KeyJolt-$version-windows-x64-Portable.zip"
    $portablePath = Join-Path $outputDirectory $portableName
    $temporaryArchive = Join-Path $outputDirectory "$([Guid]::NewGuid().ToString('N')).zip"
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    try {
        $archive = [IO.Compression.ZipFile]::Open($temporaryArchive, [IO.Compression.ZipArchiveMode]::Create)
        try {
            foreach ($entry in @(
                @{ Source = $binaryPath; Name = 'key-jolt.exe' },
                @{ Source = (Join-Path $repoRoot 'LICENSE'); Name = 'LICENSE' },
                @{ Source = (Join-Path $repoRoot 'README.md'); Name = 'README.md' }
            )) {
                [IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                    $archive, $entry.Source, $entry.Name, [IO.Compression.CompressionLevel]::Optimal
                ) | Out-Null
            }
        } finally {
            $archive.Dispose()
        }
        Move-Item -LiteralPath $temporaryArchive -Destination $portablePath -Force
    } finally {
        if (Test-Path -LiteralPath $temporaryArchive -PathType Leaf) {
            Remove-Item -LiteralPath $temporaryArchive
        }
    }
    $checksumLines = foreach ($artifact in @($installerPath, $portablePath)) {
        $digest = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
        "$digest  $([IO.Path]::GetFileName($artifact))"
    }
    $checksumPath = Join-Path $outputDirectory 'SHA256SUMS.txt'
    [IO.File]::WriteAllText($checksumPath, ($checksumLines -join "`n") + "`n", [Text.UTF8Encoding]::new($false))
    Write-Host "Portable ZIP: $portablePath"
    Write-Host "Checksum: $checksumPath"
    Write-Host 'Release packaging completed.'
} finally {
    Pop-Location
}
