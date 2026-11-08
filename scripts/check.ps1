# Everything that should pass before you tag a release (Windows PowerShell).
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
npm install;               if ($LASTEXITCODE) { exit 1 }
npm run typecheck;         if ($LASTEXITCODE) { exit 1 }
cargo test -p blockmc-core; if ($LASTEXITCODE) { exit 1 }
npm run build;             if ($LASTEXITCODE) { exit 1 }
cargo check -p blockmc;    if ($LASTEXITCODE) { exit 1 }
Write-Host "ALL CHECKS PASSED"
