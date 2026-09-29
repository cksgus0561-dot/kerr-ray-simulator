# Dot-source once: . .\env.ps1
# Prefer an existing Rust installation. The task-local fallback changes this session only.
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    $taskRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
    $taskCargo = Join-Path $taskRoot 'work\cargo'
    $taskRustup = Join-Path $taskRoot 'work\rustup'
    if (-not (Test-Path (Join-Path $taskCargo 'bin\cargo.exe'))) {
        throw 'Rust is required: https://rust-lang.org/tools/install/'
    }
    $env:CARGO_HOME = $taskCargo
    $env:RUSTUP_HOME = $taskRustup
    $env:PATH = (Join-Path $taskCargo 'bin') + ';' + $env:PATH
}
