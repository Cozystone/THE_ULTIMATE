//! XOR + popcount kernels with runtime CPU feature dispatch.
//!
//! Order of preference: AVX-512 VPOPCNTDQ > AVX2 (vpshufb nibble lookup) > hardware POPCNT >
//! portable scalar. Every path must be bit-identical to `hamming_portable` (property tested).

use std::sync::OnceLock;

/// Which kernel the dispatcher selected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelKind {
    Portable,
    Popcnt,
    Avx2,
    Avx512,
    Neon,
}

impl KernelKind {
    pub fn name(self) -> &'static str {
        match self {
            KernelKind::Portable => "portable-scalar",
            KernelKind::Popcnt => "x86-popcnt",
            KernelKind::Avx2 => "avx2-vpshufb",
            KernelKind::Avx512 => "avx512-vpopcntdq",
            KernelKind::Neon => "arm-neon",
        }
    }
}

type HamFn = unsafe fn(&[u64], &[u64]) -> u32;
type PopFn = unsafe fn(&[u64]) -> u32;

struct Dispatch {
    kind: KernelKind,
    ham: HamFn,
    pop: PopFn,
}

static DISPATCH: OnceLock<Dispatch> = OnceLock::new();

fn select() -> Dispatch {
    if std::env::var("BITMIND_KERNEL").ok().as_deref() == Some("portable") {
        return Dispatch { kind: KernelKind::Portable, ham: hamming_portable_unsafe, pop: popcount_portable_unsafe };
    }
    #[cfg(target_arch = "x86_64")]
    {
        let force = std::env::var("BITMIND_KERNEL").ok();
        let want = |k: &str| force.as_deref().map(|f| f == k).unwrap_or(true);
        if want("avx512") && is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512vpopcntdq") {
            return Dispatch { kind: KernelKind::Avx512, ham: x86::hamming_avx512, pop: x86::popcount_avx512 };
        }
        if want("avx2") && is_x86_feature_detected!("avx2") {
            return Dispatch { kind: KernelKind::Avx2, ham: x86::hamming_avx2, pop: x86::popcount_avx2 };
        }
        if want("popcnt") && is_x86_feature_detected!("popcnt") {
            return Dispatch { kind: KernelKind::Popcnt, ham: x86::hamming_popcnt, pop: x86::popcount_popcnt };
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        return Dispatch { kind: KernelKind::Neon, ham: arm::hamming_neon, pop: arm::popcount_neon };
    }
    #[allow(unreachable_code)]
    Dispatch { kind: KernelKind::Portable, ham: hamming_portable_unsafe, pop: popcount_portable_unsafe }
}

fn dispatch() -> &'static Dispatch {
    DISPATCH.get_or_init(select)
}

/// Kernel in use (for reports).
pub fn active_kernel() -> KernelKind {
    dispatch().kind
}

/// Hamming distance between two equal-length word slices.
#[inline]
pub fn hamming(a: &[u64], b: &[u64]) -> u32 {
    debug_assert_eq!(a.len(), b.len());
    // SAFETY: the selected function's CPU features were verified at dispatch time.
    unsafe { (dispatch().ham)(a, b) }
}

/// Population count of a word slice.
#[inline]
pub fn popcount(a: &[u64]) -> u32 {
    unsafe { (dispatch().pop)(a) }
}

/// Reference implementation. Every accelerated kernel must equal this.
pub fn hamming_portable(a: &[u64], b: &[u64]) -> u32 {
    let mut s = 0u32;
    for i in 0..a.len() {
        let mut x = a[i] ^ b[i];
        // SWAR popcount, independent of hardware POPCNT
        x = x - ((x >> 1) & 0x5555_5555_5555_5555);
        x = (x & 0x3333_3333_3333_3333) + ((x >> 2) & 0x3333_3333_3333_3333);
        x = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f;
        s += (x.wrapping_mul(0x0101_0101_0101_0101) >> 56) as u32;
    }
    s
}

pub fn popcount_portable(a: &[u64]) -> u32 {
    let zeros = vec![0u64; a.len()];
    hamming_portable(a, &zeros)
}

unsafe fn hamming_portable_unsafe(a: &[u64], b: &[u64]) -> u32 {
    hamming_portable(a, b)
}
unsafe fn popcount_portable_unsafe(a: &[u64]) -> u32 {
    let mut s = 0u32;
    for &w in a {
        let mut x = w;
        x = x - ((x >> 1) & 0x5555_5555_5555_5555);
        x = (x & 0x3333_3333_3333_3333) + ((x >> 2) & 0x3333_3333_3333_3333);
        x = (x + (x >> 4)) & 0x0f0f_0f0f_0f0f_0f0f;
        s += (x.wrapping_mul(0x0101_0101_0101_0101) >> 56) as u32;
    }
    s
}

/// Explicit access to every kernel available on this CPU, for cross-checking and benchmarks.
pub fn available_kernels() -> Vec<(KernelKind, fn(&[u64], &[u64]) -> u32)> {
    let mut v: Vec<(KernelKind, fn(&[u64], &[u64]) -> u32)> = vec![(KernelKind::Portable, hamming_portable)];
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("popcnt") {
            v.push((KernelKind::Popcnt, |a, b| unsafe { x86::hamming_popcnt(a, b) }));
        }
        if is_x86_feature_detected!("avx2") {
            v.push((KernelKind::Avx2, |a, b| unsafe { x86::hamming_avx2(a, b) }));
        }
        if is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512vpopcntdq") {
            v.push((KernelKind::Avx512, |a, b| unsafe { x86::hamming_avx512(a, b) }));
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        v.push((KernelKind::Neon, |a, b| unsafe { arm::hamming_neon(a, b) }));
    }
    v
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use std::arch::x86_64::*;

    #[target_feature(enable = "popcnt")]
    pub unsafe fn hamming_popcnt(a: &[u64], b: &[u64]) -> u32 {
        let mut s = 0u32;
        for i in 0..a.len() {
            s += (a[i] ^ b[i]).count_ones();
        }
        s
    }

    #[target_feature(enable = "popcnt")]
    pub unsafe fn popcount_popcnt(a: &[u64]) -> u32 {
        a.iter().map(|w| w.count_ones()).sum()
    }

    #[target_feature(enable = "avx2")]
    unsafe fn popcnt256(v: __m256i) -> __m256i {
        let lookup = _mm256_setr_epi8(
            0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4, 0, 1, 1, 2, 1, 2, 2, 3, 1, 2, 2, 3, 2, 3, 3, 4,
        );
        let low = _mm256_set1_epi8(0x0f);
        let lo = _mm256_and_si256(v, low);
        let hi = _mm256_and_si256(_mm256_srli_epi16::<4>(v), low);
        let cnt = _mm256_add_epi8(_mm256_shuffle_epi8(lookup, lo), _mm256_shuffle_epi8(lookup, hi));
        _mm256_sad_epu8(cnt, _mm256_setzero_si256())
    }

    #[target_feature(enable = "avx2")]
    unsafe fn hsum256(acc: __m256i) -> u64 {
        let mut lanes = [0u64; 4];
        _mm256_storeu_si256(lanes.as_mut_ptr() as *mut __m256i, acc);
        lanes[0] + lanes[1] + lanes[2] + lanes[3]
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn hamming_avx2(a: &[u64], b: &[u64]) -> u32 {
        let n = a.len();
        let mut acc = _mm256_setzero_si256();
        let mut i = 0;
        while i + 4 <= n {
            let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            let vb = _mm256_loadu_si256(b.as_ptr().add(i) as *const __m256i);
            acc = _mm256_add_epi64(acc, popcnt256(_mm256_xor_si256(va, vb)));
            i += 4;
        }
        let mut s = hsum256(acc);
        while i < n {
            s += (a[i] ^ b[i]).count_ones() as u64;
            i += 1;
        }
        s as u32
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn popcount_avx2(a: &[u64]) -> u32 {
        let n = a.len();
        let mut acc = _mm256_setzero_si256();
        let mut i = 0;
        while i + 4 <= n {
            let va = _mm256_loadu_si256(a.as_ptr().add(i) as *const __m256i);
            acc = _mm256_add_epi64(acc, popcnt256(va));
            i += 4;
        }
        let mut s = hsum256(acc);
        while i < n {
            s += a[i].count_ones() as u64;
            i += 1;
        }
        s as u32
    }

    #[target_feature(enable = "avx512f,avx512vpopcntdq")]
    pub unsafe fn hamming_avx512(a: &[u64], b: &[u64]) -> u32 {
        let n = a.len();
        let mut acc = _mm512_setzero_si512();
        let mut i = 0;
        while i + 8 <= n {
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i);
            let vb = _mm512_loadu_si512(b.as_ptr().add(i) as *const __m512i);
            acc = _mm512_add_epi64(acc, _mm512_popcnt_epi64(_mm512_xor_si512(va, vb)));
            i += 8;
        }
        let mut s = _mm512_reduce_add_epi64(acc) as u64;
        while i < n {
            s += (a[i] ^ b[i]).count_ones() as u64;
            i += 1;
        }
        s as u32
    }

    #[target_feature(enable = "avx512f,avx512vpopcntdq")]
    pub unsafe fn popcount_avx512(a: &[u64]) -> u32 {
        let n = a.len();
        let mut acc = _mm512_setzero_si512();
        let mut i = 0;
        while i + 8 <= n {
            let va = _mm512_loadu_si512(a.as_ptr().add(i) as *const __m512i);
            acc = _mm512_add_epi64(acc, _mm512_popcnt_epi64(va));
            i += 8;
        }
        let mut s = _mm512_reduce_add_epi64(acc) as u64;
        while i < n {
            s += a[i].count_ones() as u64;
            i += 1;
        }
        s as u32
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use std::arch::aarch64::*;

    pub unsafe fn hamming_neon(a: &[u64], b: &[u64]) -> u32 {
        let n = a.len();
        let mut total = 0u64;
        let mut i = 0;
        while i + 2 <= n {
            let va = vld1q_u64(a.as_ptr().add(i));
            let vb = vld1q_u64(b.as_ptr().add(i));
            let x = veorq_u64(va, vb);
            let c = vcntq_u8(vreinterpretq_u8_u64(x));
            total += vaddlvq_u8(c) as u64;
            i += 2;
        }
        while i < n {
            total += (a[i] ^ b[i]).count_ones() as u64;
            i += 1;
        }
        total as u32
    }

    pub unsafe fn popcount_neon(a: &[u64]) -> u32 {
        let mut total = 0u64;
        let mut i = 0;
        let n = a.len();
        while i + 2 <= n {
            let c = vcntq_u8(vreinterpretq_u8_u64(vld1q_u64(a.as_ptr().add(i))));
            total += vaddlvq_u8(c) as u64;
            i += 2;
        }
        while i < n {
            total += a[i].count_ones() as u64;
            i += 1;
        }
        total as u32
    }
}
