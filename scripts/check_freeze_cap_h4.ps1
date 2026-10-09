# capability-h4 held-out guard: learner crates must equal the frozen tag bitmind-capability-h4-v0.1.
$diff = git diff --stat bitmind-capability-h4-v0.1 -- crates/hdc-core crates/bm-relation crates/bm-memory crates/bm-agent
if ($diff) { Write-Output "FREEZE VIOLATED:"; Write-Output $diff; exit 1 } else { Write-Output "freeze check: OK (learner crates identical to bitmind-capability-h4-v0.1)" }
