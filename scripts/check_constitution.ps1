# Constitution guard: cognitive crates must not contain floating point types or LLM/embedding deps.
# Usage: powershell -File scripts/check_constitution.ps1   (exit 1 on violation)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$cognitive = @("hdc-core", "bm-relation", "bm-memory", "bm-agent")
$violations = @()
foreach ($c in $cognitive) {
    $src = Join-Path $root "crates\$c\src"
    if (-not (Test-Path $src)) { continue }
    Get-ChildItem -Recurse -Filter *.rs $src | ForEach-Object {
        $n = 0
        foreach ($line in Get-Content $_.FullName) {
            $n++
            $code = ($line -split "//")[0]
            if ($code -match "\bf(32|64)\b") { $violations += "$($_.FullName):$n float on cognitive path: $line" }
        }
    }
    $toml = Join-Path $root "crates\$c\Cargo.toml"
    $deps = Get-Content $toml -Raw
    foreach ($bad in @("candle", "tch", "ort", "onnx", "llama", "tokenizers", "openai", "anthropic", "rust-bert", "fastembed", "qdrant")) {
        if ($deps -match "(?m)^\s*$bad\s*=") { $violations += "${toml}: forbidden dependency $bad" }
    }
}
if ($violations.Count -gt 0) {
    $violations | ForEach-Object { Write-Output $_ }
    exit 1
}
Write-Output "constitution check: OK ($($cognitive -join ', '))"
