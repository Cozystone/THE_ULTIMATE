# Workstation Research Tier watchdog (PREREG-capability-h4 section 7).
# Polls learner processes (r0, phase_*, diag_*) every $Interval seconds, appends current and peak
# working set to $Log, and terminates any process whose working set exceeds $LimitGB (classified as
# a resource failure; its own logs are left untouched). Exits after $IdleCycles polls with no
# matching process.
# Usage: powershell -NoProfile -File scripts/rss_monitor.ps1 -Log <path> [-LimitGB 16] [-Interval 30]
param(
    [Parameter(Mandatory = $true)][string]$Log,
    [double]$LimitGB = 16,
    [int]$Interval = 30,
    [int]$IdleCycles = 10
)
$limit = [int64]($LimitGB * 1GB)
$idle = 0
Add-Content -Path $Log -Value "# rss_monitor start $(Get-Date -Format o) limit ${LimitGB} GB interval ${Interval} s"
while ($idle -lt $IdleCycles) {
    $procs = Get-Process -ErrorAction SilentlyContinue | Where-Object { $_.ProcessName -match '^(r0|phase_[a-z0-9]+|diag_[a-z0-9]+)$' }
    if (-not $procs) { $idle++ } else { $idle = 0 }
    foreach ($p in $procs) {
        $ws = [math]::Round($p.WorkingSet64 / 1MB)
        $peak = [math]::Round($p.PeakWorkingSet64 / 1MB)
        Add-Content -Path $Log -Value "$(Get-Date -Format o) pid $($p.Id) $($p.ProcessName) ws_mb $ws peak_mb $peak"
        if ($p.WorkingSet64 -gt $limit) {
            Add-Content -Path $Log -Value "$(Get-Date -Format o) RESOURCE FAILURE pid $($p.Id) $($p.ProcessName) ws_mb $ws > limit; terminating"
            Stop-Process -Id $p.Id -Force -Confirm:$false
        }
    }
    Start-Sleep -Seconds $Interval
}
Add-Content -Path $Log -Value "# rss_monitor end $(Get-Date -Format o)"
