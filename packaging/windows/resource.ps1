param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$Publisher,
    [Parameter(Mandatory = $true)]
    [string]$Description,
    [Parameter(Mandatory = $true)]
    [string]$TargetDirectory,
    [Parameter(Mandatory = $true)]
    [string]$IconPath,
    [string]$RcPath = ''
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function ConvertTo-ResourceString([string]$Value) {
    if ($Value -match '[\x00-\x1f\x7f]') {
        throw 'Windows resource strings must not contain control characters.'
    }
    return $Value.Replace('\', '\\').Replace('"', '\"')
}

if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$') {
    throw 'Use a stable three-part version, such as 0.1.0.'
}
$versionParts = @($Version.Split('.') | ForEach-Object { [UInt16]::Parse($_) }) + @(0)
$fixedVersion = $versionParts -join ','
$stringVersion = $versionParts -join '.'
$icon = (Resolve-Path -LiteralPath $IconPath).Path

if ([string]::IsNullOrWhiteSpace($RcPath)) {
    $compiler = Get-Command 'rc.exe' -CommandType Application -ErrorAction SilentlyContinue
    if ($null -ne $compiler) {
        $RcPath = $compiler.Source
    } else {
        $sdkBin = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
        if (Test-Path -LiteralPath $sdkBin -PathType Container) {
            $sdkVersions = Get-ChildItem -LiteralPath $sdkBin -Directory |
                Where-Object { $_.Name -match '^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$' } |
                Sort-Object { [Version]$_.Name } -Descending
            foreach ($sdkVersion in $sdkVersions) {
                $candidate = Join-Path $sdkVersion.FullName 'x64\rc.exe'
                if (Test-Path -LiteralPath $candidate -PathType Leaf) {
                    $RcPath = $candidate
                    break
                }
            }
        }
    }
}
if ([string]::IsNullOrWhiteSpace($RcPath) -or -not (Test-Path -LiteralPath $RcPath -PathType Leaf)) {
    throw 'Install a Windows SDK with rc.exe, or pass -RcPath with its full path.'
}

$resourceIcon = ConvertTo-ResourceString $icon
$resourcePublisher = ConvertTo-ResourceString $Publisher
$resourceDescription = ConvertTo-ResourceString $Description
$source = @"
#pragma code_page(65001)
LANGUAGE 9, 1
1 ICON "$resourceIcon"
1 VERSIONINFO
FILEVERSION $fixedVersion
PRODUCTVERSION $fixedVersion
FILEFLAGSMASK 0x3f
FILEFLAGS 0
FILEOS 0x40004
FILETYPE 1
FILESUBTYPE 0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName", "$resourcePublisher\0"
            VALUE "FileDescription", "$resourceDescription\0"
            VALUE "FileVersion", "$stringVersion\0"
            VALUE "InternalName", "key-jolt\0"
            VALUE "OriginalFilename", "key-jolt.exe\0"
            VALUE "ProductName", "KeyJolt\0"
            VALUE "ProductVersion", "$stringVersion\0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"@

$iconDigest = (Get-FileHash -LiteralPath $icon -Algorithm SHA256).Hash
$hasher = [Security.Cryptography.SHA256]::Create()
try {
    $digestBytes = $hasher.ComputeHash([Text.Encoding]::UTF8.GetBytes("$source`n$iconDigest"))
    $resourceDigest = [BitConverter]::ToString($digestBytes).Replace('-', '').ToLowerInvariant()
} finally {
    $hasher.Dispose()
}
$resourceDirectory = Join-Path $TargetDirectory 'keyjolt-resources\windows-x64'
New-Item -ItemType Directory -Force -Path $resourceDirectory | Out-Null
$sourcePath = Join-Path $resourceDirectory "key-jolt-$resourceDigest.rc"
$resourcePath = Join-Path $resourceDirectory "key-jolt-$resourceDigest.res"
[IO.File]::WriteAllText($sourcePath, $source, [Text.UTF8Encoding]::new($false))
& $RcPath /nologo /fo $resourcePath $sourcePath | Out-Host
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $resourcePath -PathType Leaf)) {
    throw 'Windows icon/version resource compilation failed.'
}
return $resourcePath
