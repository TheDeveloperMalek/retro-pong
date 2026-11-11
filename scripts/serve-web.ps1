param([int]$Port = 8000)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$WebDir = Join-Path $Root "web-dist"

if (-not (Test-Path (Join-Path $WebDir "index.html"))) {
    throw "web-dist/index.html was not found. Run .\scripts\build-web.ps1 first."
}

$WebFontPath = Join-Path $WebDir "assets/fonts/NotoSansMono-VF.ttf"
if (-not (Test-Path $WebFontPath) -or (Get-Item $WebFontPath).Length -lt 10000) {
    throw "The WebAssembly font is missing from web-dist. Run .\scripts\build-web.ps1 again."
}

Set-Location $WebDir
if (Get-Command py -ErrorAction SilentlyContinue) {
    & py -3 -m http.server $Port --bind 127.0.0.1
    exit $LASTEXITCODE
}
if (Get-Command python -ErrorAction SilentlyContinue) {
    & python -m http.server $Port --bind 127.0.0.1
    exit $LASTEXITCODE
}
throw "Python 3 was not found. Install Python 3 or run: python -m http.server $Port --directory `"$WebDir`""
