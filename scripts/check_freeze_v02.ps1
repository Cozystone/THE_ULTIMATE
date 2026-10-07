# v0.2 held-out guard: the learner crates must be identical to the frozen v0.2 tag.
$diff = git diff --stat bitmind-v0.2 -- crates/hdc-core crates/bm-relation crates/bm-memory crates/bm-agent
if ($diff) { Write-Output "FREEZE VIOLATED:"; Write-Output $diff; exit 1 } else { Write-Output "freeze check: OK (learner crates identical to bitmind-v0.2)" }
