$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    . .\env.ps1
    cargo fmt
    if ($LASTEXITCODE -ne 0) { throw 'cargo fmt failed' }
    cargo clippy --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'cargo clippy failed' }
    cargo test
    if ($LASTEXITCODE -ne 0) { throw 'cargo test failed' }
    cargo test --release
    if ($LASTEXITCODE -ne 0) { throw 'cargo test --release failed' }
} finally { Pop-Location }
