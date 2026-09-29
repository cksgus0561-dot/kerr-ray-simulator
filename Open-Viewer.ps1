param([string]$InputRun = 'results/visualization_4096', [string]$Config)
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $viewer = Join-Path $PSScriptRoot 'bin/kerr-viewer.exe'
    if (-not (Test-Path -LiteralPath $viewer)) {
        . (Join-Path $PSScriptRoot 'env.ps1')
        cargo build --release --offline --bin kerr-viewer
        if ($LASTEXITCODE -ne 0) { throw 'Viewer build failed' }
        $viewer = Join-Path $PSScriptRoot 'target/release/kerr-viewer.exe'
    }
    if ($Config) { & $viewer --config $Config }
    else { & $viewer --input $InputRun }
    if ($LASTEXITCODE -ne 0) { throw 'Viewer failed; inspect the adapter/error log above' }
}
finally { Pop-Location }
