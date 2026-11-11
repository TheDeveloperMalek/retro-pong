$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

function Invoke-CheckedCommand {
    param(
        [Parameter(Mandatory = $true)][string]$Command,
        [Parameter(Mandatory = $true)][string[]]$Arguments
    )
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed ($LASTEXITCODE): $Command $($Arguments -join ' ')"
    }
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "A required build command was not found. Install the project prerequisites and reopen PowerShell."
}
if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
    throw "A required platform-target manager was not found. Install the project prerequisites and reopen PowerShell."
}

# Browsers do not expose a system-font directory to WebAssembly. Bundle a font
# asset so the game's text rendering can render consistently across all supported locales.
$FontDirectory = Join-Path $Root "assets/fonts"
$FontPath = Join-Path $FontDirectory "NotoSansMono-VF.ttf"
$FontLicensePath = Join-Path $FontDirectory "OFL.txt"
$FontUrl = "https://raw.githubusercontent.com/google/fonts/main/ofl/notosansmono/NotoSansMono%5Bwdth%2Cwght%5D.ttf"
$FontLicenseUrl = "https://raw.githubusercontent.com/google/fonts/main/ofl/notosansmono/OFL.txt"
New-Item -ItemType Directory -Path $FontDirectory -Force | Out-Null

if (-not (Test-Path $FontPath) -or (Get-Item $FontPath).Length -lt 10000) {
    Write-Host "Downloading Noto Sans Mono for WebAssembly text rendering..." -ForegroundColor Cyan
    $FontTemp = "$FontPath.tmp"
    Invoke-WebRequest -UseBasicParsing -Uri $FontUrl -OutFile $FontTemp
    if (-not (Test-Path $FontTemp) -or (Get-Item $FontTemp).Length -lt 10000) {
        Remove-Item $FontTemp -Force -ErrorAction SilentlyContinue
        throw "The Noto Sans Mono font download is missing or incomplete."
    }
    Move-Item $FontTemp $FontPath -Force
}
if (-not (Test-Path $FontLicensePath) -or (Get-Item $FontLicensePath).Length -eq 0) {
    Write-Host "Downloading the Noto Sans Mono OFL license..." -ForegroundColor Cyan
    $LicenseTemp = "$FontLicensePath.tmp"
    Invoke-WebRequest -UseBasicParsing -Uri $FontLicenseUrl -OutFile $LicenseTemp
    if (-not (Test-Path $LicenseTemp) -or (Get-Item $LicenseTemp).Length -lt 1000) {
        Remove-Item $LicenseTemp -Force -ErrorAction SilentlyContinue
        throw "The Noto Sans Mono license download is missing or incomplete."
    }
    Move-Item $LicenseTemp $FontLicensePath -Force
}

$InstalledTargets = rustup target list --installed
if ($LASTEXITCODE -ne 0) { throw "Could not query installed browser targets." }
if ($InstalledTargets -notcontains "wasm32-unknown-unknown") {
    Invoke-CheckedCommand -Command "rustup" -Arguments @("target", "add", "wasm32-unknown-unknown")
}

if (-not (Test-Path "Cargo.lock")) {
    Invoke-CheckedCommand -Command "cargo" -Arguments @("generate-lockfile")
}

$InsideWasmBindgenPackage = $false
$ExpectedBindgenVersion = $null
foreach ($Line in Get-Content "Cargo.lock") {
    if ($Line -eq "[[package]]") {
        $InsideWasmBindgenPackage = $false
        continue
    }
    if ($Line -eq 'name = "wasm-bindgen"') {
        $InsideWasmBindgenPackage = $true
        continue
    }
    if ($InsideWasmBindgenPackage -and $Line -match '^version = "([^\"]+)"$') {
        $ExpectedBindgenVersion = $Matches[1]
        break
    }
}
if (-not $ExpectedBindgenVersion) {
    throw "Could not find wasm-bindgen in Cargo.lock. Check the dependency tree."
}

$InstalledBindgenVersion = $null
if (Get-Command wasm-bindgen -ErrorAction SilentlyContinue) {
    $VersionOutput = wasm-bindgen --version
    if ($LASTEXITCODE -eq 0 -and $VersionOutput -match '(\d+\.\d+\.\d+)') {
        $InstalledBindgenVersion = $Matches[1]
    }
}
if ($InstalledBindgenVersion -ne $ExpectedBindgenVersion) {
    Write-Host "Installing matching wasm-bindgen-cli $ExpectedBindgenVersion (current: $InstalledBindgenVersion)..." -ForegroundColor Yellow
    Invoke-CheckedCommand -Command "cargo" -Arguments @(
        "install", "wasm-bindgen-cli",
        "--version", $ExpectedBindgenVersion,
        "--locked", "--force"
    )
}

if (Test-Path "web-dist") { Remove-Item "web-dist" -Recurse -Force }
New-Item -ItemType Directory -Path "web-dist/pkg/webgl2" -Force | Out-Null
New-Item -ItemType Directory -Path "web-dist/pkg/webgpu" -Force | Out-Null

function Build-WebRenderer {
    param(
        [Parameter(Mandatory = $true)][string]$Renderer,
        [string[]]$CargoFeatures = @()
    )

    Write-Host "Building Retro Pong WebAssembly renderer: $Renderer" -ForegroundColor Cyan
    $RendererTargetDir = Join-Path $Root "target/web-$Renderer"
    $PreviousTargetDir = $env:CARGO_TARGET_DIR
    try {
        $env:CARGO_TARGET_DIR = $RendererTargetDir
        $BuildArgs = @("build", "--release", "--target", "wasm32-unknown-unknown")
        if ($CargoFeatures.Count -gt 0) { $BuildArgs += $CargoFeatures }
        Invoke-CheckedCommand -Command "cargo" -Arguments $BuildArgs
    }
    finally {
        if ($null -eq $PreviousTargetDir) {
            Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue
        }
        else {
            $env:CARGO_TARGET_DIR = $PreviousTargetDir
        }
    }

    $WasmDirectory = Join-Path $RendererTargetDir "wasm32-unknown-unknown/release"
    $WasmArtifact = Get-ChildItem -Path $WasmDirectory -Filter "*.wasm" -File | Select-Object -First 1
    if (-not $WasmArtifact) {
        throw "Could not find a compiled .wasm executable for renderer '$Renderer'."
    }

    $OutputDir = Join-Path $Root "web-dist/pkg/$Renderer"
    Invoke-CheckedCommand -Command "wasm-bindgen" -Arguments @(
        "--target", "web",
        "--out-dir", $OutputDir,
        "--out-name", "retro_pong",
        $WasmArtifact.FullName
    )

    if (-not (Test-Path (Join-Path $OutputDir "retro_pong.js")) -or
        -not (Test-Path (Join-Path $OutputDir "retro_pong_bg.wasm"))) {
        throw "WebAssembly output for '$Renderer' is incomplete."
    }
}

Build-WebRenderer -Renderer "webgl2"
Build-WebRenderer -Renderer "webgpu" -CargoFeatures @("--no-default-features", "--features", "renderer-webgpu")

Copy-Item "web/index.html" "web-dist/index.html"
Copy-Item "web/style.css" "web-dist/style.css"
New-Item -ItemType Directory -Path "web-dist/assets" -Force | Out-Null
Copy-Item "assets/*" "web-dist/assets" -Recurse -Force
$BundledFont = Join-Path $Root "web-dist/assets/fonts/NotoSansMono-VF.ttf"
if (-not (Test-Path $BundledFont) -or (Get-Item $BundledFont).Length -lt 10000) {
    throw "The bundled WebAssembly font is missing from web-dist/assets/fonts."
}
New-Item -ItemType File -Path "web-dist/.nojekyll" -Force | Out-Null

Write-Host "Web build completed: $Root/web-dist" -ForegroundColor Green
Write-Host "Generated renderers: WebGL2 (preferred) and WebGPU (fallback)."
Write-Host "Run .\scripts\serve-web.ps1 to preview it locally."
