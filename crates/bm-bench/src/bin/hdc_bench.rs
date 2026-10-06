//! Phase A baseline: kernel latency, bundling, cleanup search scaling, capacity and noise curves.
//! Writes experiments/results/hdc_bench.json. Floats are used here only for reporting.

use hdc_core::kernel::{active_kernel, available_kernels};
use hdc_core::*;
use std::hint::black_box;
use std::time::Instant;

fn ns_per<F: FnMut()>(iters: u64, mut f: F) -> f64 {
    for _ in 0..iters.min(1000) {
        f();
    }
    let t = Instant::now();
    for _ in 0..iters {
        f();
    }
    t.elapsed().as_nanos() as f64 / iters as f64
}

fn kernel_rows<const W: usize>(label: &str, out: &mut Vec<String>) {
    let mut rng = Rng::new(1);
    let a = Hv::<W>::random(&mut rng);
    let b = Hv::<W>::random(&mut rng);
    for (kind, f) in available_kernels() {
        let ns = ns_per(2_000_000, || {
            black_box(f(black_box(&a.w), black_box(&b.w)));
        });
        println!("  {label:>6} hamming {:<18} {:>8.1} ns", kind.name(), ns);
        out.push(format!("{{\"dim\":\"{label}\",\"op\":\"hamming\",\"kernel\":\"{}\",\"ns\":{:.2}}}", kind.name(), ns));
    }
    let ns = ns_per(2_000_000, || {
        black_box(black_box(&a).bind(black_box(&b)));
    });
    println!("  {label:>6} bind (xor)                 {ns:>8.1} ns");
    out.push(format!("{{\"dim\":\"{label}\",\"op\":\"bind\",\"ns\":{ns:.2}}}"));
    let mut k = 0i64;
    let ns = ns_per(1_000_000, || {
        k = (k + 7919) % 16384;
        black_box(black_box(&a).permute(k));
    });
    println!("  {label:>6} permute (arbitrary k)      {ns:>8.1} ns");
    out.push(format!("{{\"dim\":\"{label}\",\"op\":\"permute\",\"ns\":{ns:.2}}}"));
    let mut bp = BitPlaneBundler::<W>::new();
    let ns = ns_per(200_000, || {
        bp.add(black_box(&a));
    });
    println!("  {label:>6} bundle add (bit-plane)     {ns:>8.1} ns  (planes={})", bp.planes());
    out.push(format!("{{\"dim\":\"{label}\",\"op\":\"bundle_add_bitplane\",\"ns\":{ns:.2}}}"));
    let mut cb = CounterBundler::<W>::new();
    let ns = ns_per(20_000, || {
        cb.add(black_box(&a));
    });
    println!("  {label:>6} bundle add (i32 counter)   {ns:>8.1} ns");
    out.push(format!("{{\"dim\":\"{label}\",\"op\":\"bundle_add_counter\",\"ns\":{ns:.2}}}"));
}

fn cleanup_rows<const W: usize>(label: &str, sizes: &[usize], out: &mut Vec<String>) {
    let mut rng = Rng::new(2);
    let max = *sizes.iter().max().unwrap();
    let mut mem = CleanupMemory::<W>::new();
    let mut probe = Vec::new();
    let mut done = 0usize;
    for &n in sizes {
        while done < n {
            let h = Hv::<W>::random(&mut rng);
            if done % (max / 16).max(1) == 0 {
                probe.push((done as u64, h.clone()));
            }
            mem.insert(done as u64, &h);
            done += 1;
        }
        let mut qs: Vec<(u64, Hv<W>)> = probe.clone();
        for (_, q) in qs.iter_mut() {
            q.flip_random(W * 64 * 25 / 100, &mut rng);
        }
        let reps = (2_000_000 / n).clamp(3, 2000);
        let t = Instant::now();
        let mut ok = 0;
        for r in 0..reps {
            let (id, q) = &qs[r % qs.len()];
            if mem.cleanup(q).map(|h| h.id) == Some(*id) {
                ok += 1;
            }
        }
        let us = t.elapsed().as_nanos() as f64 / reps as f64 / 1000.0;
        let mb = mem.bytes() as f64 / 1e6;
        println!("  {label:>6} cleanup exact N={n:>8} {us:>10.1} us/query  mem {mb:>8.1} MB  recall@25%noise {ok}/{reps}");
        out.push(format!(
            "{{\"dim\":\"{label}\",\"op\":\"cleanup_exact\",\"n\":{n},\"us_per_query\":{us:.2},\"mem_mb\":{mb:.2},\"ok\":{ok},\"reps\":{reps}}}"
        ));
        for (name, mode) in [("prefix", 1u8), ("par16", 2u8)] {
            for noise in [25usize, 40] {
                let mut qs2: Vec<(u64, Hv<W>)> = probe.clone();
                for (_, q) in qs2.iter_mut() {
                    q.flip_random(W * 64 * noise / 100, &mut rng);
                }
                let reps2 = (20_000_000 / n).clamp(16, 4000);
                let t = Instant::now();
                let mut ok2 = 0;
                for r in 0..reps2 {
                    let (id, q) = &qs2[r % qs2.len()];
                    let h = if mode == 1 { mem.cleanup_fast(q) } else { mem.cleanup_par(q, 16) };
                    if h.map(|h| h.id) == Some(*id) {
                        ok2 += 1;
                    }
                }
                let us2 = t.elapsed().as_nanos() as f64 / reps2 as f64 / 1000.0;
                println!("  {label:>6} cleanup {name:<6}N={n:>8} {us2:>10.1} us/query  recall@{noise}%noise {ok2}/{reps2}");
                out.push(format!(
                    "{{\"dim\":\"{label}\",\"op\":\"cleanup_{name}\",\"n\":{n},\"noise\":{noise},\"us_per_query\":{us2:.2},\"ok\":{ok2},\"reps\":{reps2}}}"
                ));
            }
        }
    }
}

fn capacity_rows<const W: usize>(label: &str, ks: &[usize], out: &mut Vec<String>) {
    let mut rng = Rng::new(3);
    let n = 10_000usize;
    let items: Vec<Hv<W>> = (0..n).map(|_| Hv::<W>::random(&mut rng)).collect();
    for &k in ks {
        let mut hit = 0usize;
        let trials = 3;
        for _ in 0..trials {
            let idx = rng.sample_distinct(n, k);
            let mut b = BitPlaneBundler::<W>::new();
            for &i in &idx {
                b.add(&items[i]);
            }
            let s = b.majority(&Hv::<W>::random(&mut rng));
            let mut d: Vec<(u32, usize)> = items.iter().enumerate().map(|(i, h)| (h.distance(&s), i)).collect();
            d.sort();
            let top: std::collections::HashSet<usize> = d[..k].iter().map(|x| x.1).collect();
            hit += idx.iter().filter(|i| top.contains(i)).count();
        }
        let recall = hit as f64 / (k * trials) as f64;
        println!("  {label:>6} capacity k={k:>5}  recall {recall:.4}");
        out.push(format!("{{\"dim\":\"{label}\",\"op\":\"capacity\",\"k\":{k},\"recall\":{recall:.4}}}"));
    }
}

fn noise_rows<const W: usize>(label: &str, out: &mut Vec<String>) {
    let mut rng = Rng::new(4);
    let n = 10_000u64;
    let mut mem = CleanupMemory::<W>::new();
    let items: Vec<Hv<W>> = (0..n).map(|_| Hv::<W>::random(&mut rng)).collect();
    for (i, h) in items.iter().enumerate() {
        mem.insert(i as u64, h);
    }
    for pct in [10usize, 20, 30, 35, 40, 42, 44, 46, 48] {
        let (mut ok, mut wrong, mut abst) = (0, 0, 0);
        for t in 0..100usize {
            let id = (t * 97) % n as usize;
            let mut q = items[id].clone();
            q.flip_random(W * 64 * pct / 100, &mut rng);
            match mem.cleanup(&q) {
                Some(h) if h.id == id as u64 => ok += 1,
                Some(_) => wrong += 1,
                None => abst += 1,
            }
        }
        println!("  {label:>6} noise {pct:>2}%  correct {ok:>3}  wrong {wrong:>3}  abstain {abst:>3}");
        out.push(format!(
            "{{\"dim\":\"{label}\",\"op\":\"noise\",\"pct\":{pct},\"correct\":{ok},\"wrong\":{wrong},\"abstain\":{abst}}}"
        ));
    }
}

fn main() {
    let mut out = Vec::new();
    println!("BITMIND hdc_bench  active kernel: {}", active_kernel().name());
    println!("[kernels]");
    kernel_rows::<256>("16k", &mut out);
    kernel_rows::<512>("32k", &mut out);
    println!("[cleanup search, single thread]");
    cleanup_rows::<256>("16k", &[1_000, 10_000, 100_000, 1_000_000], &mut out);
    cleanup_rows::<512>("32k", &[1_000, 10_000, 100_000], &mut out);
    println!("[bundle capacity, 10k-item codebook]");
    capacity_rows::<256>("16k", &[100, 200, 300, 400, 600, 800, 1600], &mut out);
    capacity_rows::<512>("32k", &[200, 400, 600, 800, 1200, 1600, 3200], &mut out);
    println!("[noise recovery, 10k items, z=6 floor]");
    noise_rows::<256>("16k", &mut out);
    noise_rows::<512>("32k", &mut out);
    let json = format!(
        "{{\"kernel\":\"{}\",\"rows\":[\n{}\n]}}",
        active_kernel().name(),
        out.join(",\n")
    );
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/hdc_bench.json");
    std::fs::write(&path, json).expect("write results");
    println!("saved {}", path.display());
}
