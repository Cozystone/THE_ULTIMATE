//! BITMIND evaluation harness. Floating point is allowed here only (reporting).

pub mod baselines;

use bm_relation::RelationEngine;

/// D049: declare a world's ordinal sensor channels to an engine.
pub fn declare(rel: &mut RelationEngine, ordinal: &[u16]) {
    for &c in ordinal {
        rel.declare_ordinal(c);
    }
}

/// Peak working set of this process in MB (Windows; None elsewhere or on failure).
pub fn peak_working_set_mb() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).PeakWorkingSet64")])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().ok().map(|b| b / 1048576.0)
}

/// One line every evaluation binary prints at exit (pre-registered measurement, v0.2).
pub fn resources_line(start: std::time::Instant) -> String {
    format!(
        "resources: wall {:.1} s, peak working set {} MB",
        start.elapsed().as_secs_f64(),
        peak_working_set_mb().map(|m| format!("{m:.0}")).unwrap_or_else(|| "n/a".to_string())
    )
}

/// Current working set of this process in MB (Windows; None elsewhere or on failure).
pub fn working_set_mb() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).WorkingSet64")])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().ok().map(|b| b / 1048576.0)
}

/// v0.4 development report: with BM_STATS set, prints one line per seed when dropped (at the end
/// of the seed's loop iteration, after the seed's engines): wall time, current and peak working set.
pub struct SeedGuard {
    seed: u64,
    start: std::time::Instant,
}

impl SeedGuard {
    pub fn new(seed: u64) -> Self {
        if std::env::var_os("BM_STATS").is_some() {
            eprintln!("SEED_START {seed}");
        }
        SeedGuard { seed, start: std::time::Instant::now() }
    }
}

impl Drop for SeedGuard {
    fn drop(&mut self) {
        if std::env::var_os("BM_STATS").is_some() {
            eprintln!(
                "SEED_END {} wall {:.1} s working set {} MB peak {} MB",
                self.seed,
                self.start.elapsed().as_secs_f64(),
                working_set_mb().map(|m| format!("{m:.0}")).unwrap_or_else(|| "n/a".into()),
                peak_working_set_mb().map(|m| format!("{m:.0}")).unwrap_or_else(|| "n/a".into())
            );
        }
    }
}
