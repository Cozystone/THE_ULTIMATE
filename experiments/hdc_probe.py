"""
HDC probe: Gemini 제안(능동추론 + 계층형 HDC 월드모델)의 검증 가능한 주장 4가지를 실측한다.
  1. 용량: 10,000비트 하이퍼벡터 하나에 몇 개의 항목을 겹쳐 담을 수 있나 (= 간섭에 의한 망각 곡선)
  2. 계층: 중간 정리(cleanup) 없이 구조를 몇 단계 중첩할 수 있나
  3. 일반화: 학습된 인코더 없이 처음 보는 입력에 일반화하나
  4. 비용: 10,000비트 해밍 검색이 실수 임베딩 검색보다 실제로 싼가
실행: python hdc_probe.py  (numpy>=2.0, torch 선택)
"""
import json, os, time
from concurrent.futures import ThreadPoolExecutor
import numpy as np

rng = np.random.default_rng(42)
D = 10_000
OUT = {}


def rand_hv(n, d=D):
    return rng.integers(0, 2, size=(n, d), dtype=np.uint8)


def bundle(X):
    s = X.sum(axis=0, dtype=np.int32)
    k = X.shape[0]
    out = (2 * s > k).astype(np.uint8)
    ties = 2 * s == k
    if ties.any():
        out[ties] = rng.integers(0, 2, size=int(ties.sum()), dtype=np.uint8)
    return out


def pk(M):
    return np.packbits(M, axis=-1)


def sim(qp, Mp, d=D):
    dist = np.bitwise_count(np.bitwise_xor(Mp, qp)).sum(axis=-1, dtype=np.int64)
    return 1.0 - dist / d


# ---------------------------------------------------------------- 1. capacity
def exp_capacity():
    N = 10_000
    C = rand_hv(N)
    Cp = pk(C)
    rows = []
    for k in [10, 50, 100, 200, 400, 800, 1600, 3200]:
        rec = []
        for _ in range(5):
            idx = rng.choice(N, size=k, replace=False)
            s = sim(pk(bundle(C[idx])), Cp)
            top = np.argpartition(-s, k)[:k]
            rec.append(len(set(top.tolist()) & set(idx.tolist())) / k)
        rows.append({"items_bundled": k, "recall": round(float(np.mean(rec)), 4)})
        print(f"[capacity] k={k:5d} recall={np.mean(rec):.3f}")
    return rows


# ---------------------------------------------------------------- 2. hierarchy
def exp_hierarchy():
    F = rand_hv(10_000)
    Fp = pk(F)
    roles = rand_hv(16)  # 0=target slot, 1=child slot, 2..=distractor slots
    rows = []
    for m in [3, 5, 9]:
        for L in range(1, 7):
            ok, T = 0, 100
            for _ in range(T):
                tgt = int(rng.integers(F.shape[0]))
                pairs = [roles[0] ^ F[tgt]] + [
                    roles[2 + j] ^ F[o]
                    for j, o in enumerate(rng.choice(F.shape[0], m - 1, replace=False))
                ]
                rec = bundle(np.stack(pairs))
                for _l in range(2, L + 1):
                    pairs = [roles[1] ^ rec] + [
                        roles[2 + j] ^ F[o]
                        for j, o in enumerate(rng.choice(F.shape[0], m - 1, replace=False))
                    ]
                    rec = bundle(np.stack(pairs))
                q = rec
                for _l in range(L, 1, -1):
                    q = q ^ roles[1]
                q = q ^ roles[0]
                ok += int(np.argmax(sim(pk(q), Fp)) == tgt)
            rows.append({"slots_per_level": m, "depth": L, "top1_acc": ok / T})
            print(f"[hierarchy] m={m} depth={L} acc={ok / T:.2f}")
    return rows


# ---------------------------------------------------------------- 3a. time-of-day generalization
def circular_levels(nbins, d=D):
    half = nbins // 2
    base = rng.integers(0, 2, size=d, dtype=np.uint8)
    perm = rng.permutation(d)
    step = d // (2 * half)
    chunks = [perm[i * step:(i + 1) * step] for i in range(half)]
    V = np.empty((nbins, d), dtype=np.uint8)
    for i in range(nbins):
        v = base.copy()
        for c in (chunks[:i] if i <= half else chunks[i - half:]):
            v[c] ^= 1
        V[i] = v
    return V


def activity(h):
    if h < 7 or h >= 23:
        return 0  # sleep
    if h < 9:
        return 1  # morning routine
    if 12 <= h < 13 or 18 <= h < 19.5:
        return 3  # meal
    if h < 18:
        return 2  # work
    return 4  # leisure


def proto_predict(Xtr, ytr, Xte, ncls):
    P = []
    for c in range(ncls):
        sel = Xtr[ytr == c]
        P.append(bundle(sel) if len(sel) else rng.integers(0, 2, D, dtype=np.uint8))
    Pp = pk(np.stack(P))
    return np.array([int(np.argmax(sim(pk(x), Pp))) for x in Xte])


def exp_time_generalization():
    nb = 96
    y = np.array([activity(i / 4) for i in range(nb)])
    res = {"atomic_random_codes": [], "hand_designed_circular_codes": []}
    for _ in range(10):
        tr = rng.choice(nb, size=int(nb * 0.3), replace=False)
        te = np.setdiff1d(np.arange(nb), tr)
        for name, V in [("atomic_random_codes", rand_hv(nb)),
                        ("hand_designed_circular_codes", circular_levels(nb))]:
            pred = proto_predict(V[tr], y[tr], V[te], 5)
            res[name].append(float((pred == y[te]).mean()))
    out = {k: round(float(np.mean(v)), 3) for k, v in res.items()}
    out["majority_class_baseline"] = round(float(np.bincount(y).max() / nb), 3)
    print(f"[time-of-day] {out}")
    return out


# ---------------------------------------------------------------- 3b. vision without a learned encoder
SHAPES = {
    "plus": [(0, 1), (1, 0), (1, 1), (1, 2), (2, 1)],
    "x": [(0, 0), (0, 2), (1, 1), (2, 0), (2, 2)],
    "hline": [(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
    "vline": [(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
}
S = 16


def make_images(n):
    X, y = [], []
    for ci, pts in enumerate(SHAPES.values()):
        h = max(p[0] for p in pts) + 1
        w = max(p[1] for p in pts) + 1
        for _ in range(n):
            r = rng.integers(0, S - h + 1)
            c = rng.integers(0, S - w + 1)
            img = np.zeros((S, S), np.uint8)
            for a, b in pts:
                img[r + a, c + b] = 1
            X.append(img)
            y.append(ci)
    return np.array(X), np.array(y)


def hdc_encode_images(X, P, V0, V1):
    # standard HDC "position x value" record encoding: bundle_i (Pos_i XOR Val(x_i))
    A0 = (P ^ V0).astype(np.float32)
    A1 = (P ^ V1).astype(np.float32)
    counts = A0.sum(0)[None, :] + X.reshape(len(X), -1).astype(np.float32) @ (A1 - A0)
    n = P.shape[0]
    bits = (2 * counts > n).astype(np.uint8)
    ties = 2 * counts == n
    bits[ties] = rng.integers(0, 2, size=int(ties.sum()), dtype=np.uint8)
    return bits


def exp_vision():
    Xtr, ytr = make_images(300)
    Xte, yte = make_images(300)
    P = rand_hv(S * S)
    V0, V1 = rand_hv(1)[0], rand_hv(1)[0]
    Htr = hdc_encode_images(Xtr, P, V0, V1)
    Hte = hdc_encode_images(Xte, P, V0, V1)
    out = {"chance": 0.25}
    out["hdc_prototype"] = round(float((proto_predict(Htr, ytr, Hte, 4) == yte).mean()), 3)
    # HDC with perceptron-style retraining (standard trick to improve HDC classifiers)
    Btr = (2.0 * Htr - 1).astype(np.float32)
    Bte = (2.0 * Hte - 1).astype(np.float32)
    W = np.stack([Btr[ytr == c].sum(0) for c in range(4)])
    for _ in range(30):
        for i in rng.permutation(len(Btr)):
            p = int(np.argmax(W @ Btr[i] / (np.linalg.norm(W, axis=1) + 1e-9)))
            if p != ytr[i]:
                W[ytr[i]] += Btr[i]
                W[p] -= Btr[i]
    pred = np.argmax(Bte @ W.T / (np.linalg.norm(W, axis=1) + 1e-9), axis=1)
    out["hdc_retrained_30_epochs"] = round(float((pred == yte).mean()), 3)
    try:
        import torch
        import torch.nn as nn
        torch.manual_seed(0)
        dev = "cuda" if torch.cuda.is_available() else "cpu"
        net = nn.Sequential(nn.Conv2d(1, 16, 3, padding=1), nn.ReLU(),
                            nn.Conv2d(16, 32, 3, padding=1), nn.ReLU(),
                            nn.AdaptiveMaxPool2d(1), nn.Flatten(), nn.Linear(32, 4)).to(dev)
        opt = torch.optim.Adam(net.parameters(), lr=1e-2)
        xt = torch.tensor(Xtr[:, None], dtype=torch.float32, device=dev)
        yt = torch.tensor(ytr, device=dev)
        for _ in range(60):
            opt.zero_grad()
            loss = nn.functional.cross_entropy(net(xt), yt)
            loss.backward()
            opt.step()
        with torch.no_grad():
            pr = net(torch.tensor(Xte[:, None], dtype=torch.float32, device=dev)).argmax(1).cpu().numpy()
        out["tiny_cnn_learned_encoder"] = round(float((pr == yte).mean()), 3)
        out["tiny_cnn_params"] = int(sum(p.numel() for p in net.parameters()))
    except Exception as e:  # torch optional
        out["tiny_cnn_learned_encoder"] = f"skipped: {e}"
    print(f"[vision] {out}")
    return out


# ---------------------------------------------------------------- 4. cost
def timed(fn, reps=3):
    fn()
    best = 1e9
    for _ in range(reps):
        t = time.perf_counter()
        fn()
        best = min(best, time.perf_counter() - t)
    return best


def threaded_hamming(M, q, workers=16):
    parts = np.array_split(np.arange(len(M)), workers)

    def run(ix):
        sl = M[ix[0]:ix[-1] + 1]
        return np.bitwise_count(sl ^ q).sum(axis=1, dtype=np.int32)

    with ThreadPoolExecutor(workers) as ex:
        return np.concatenate(list(ex.map(run, parts)))


def exp_cost():
    N = 1_000_000
    rows = []

    def add(name, nbytes, sec):
        rows.append({"method": name, "memory_MB": round(nbytes / 1e6, 1), "search_ms": round(sec * 1e3, 1)})
        print(f"[cost] {name:46s} mem={nbytes / 1e6:8.1f}MB  search={sec * 1e3:8.1f}ms")

    M = rng.integers(0, np.iinfo(np.uint64).max, size=(N, 160), dtype=np.uint64, endpoint=True)  # 10,240 bits
    q = M[123].copy()
    add("HDC 10,240-bit Hamming, CPU 16 threads", M.nbytes, timed(lambda: threaded_hamming(M, q)))
    del M
    B = rng.integers(0, np.iinfo(np.uint64).max, size=(N, 6), dtype=np.uint64, endpoint=True)  # 384 bits
    qb = B[123].copy()
    add("binary-quantized 384-d embedding, CPU 16 threads", B.nbytes, timed(lambda: threaded_hamming(B, qb)))
    del B
    E = rng.standard_normal((N, 384), dtype=np.float32)
    qe = E[123].copy()
    add("float32 384-d embedding dot product, CPU", E.nbytes, timed(lambda: E @ qe))
    del E
    try:
        import torch
        if torch.cuda.is_available():
            G = torch.randn(N, 384, device="cuda", dtype=torch.float16)
            qg = G[123].clone()

            def g():
                r = G @ qg
                torch.cuda.synchronize()
                return r

            add("float16 384-d embedding dot product, GPU", G.numel() * 2, timed(g))
            del G
            torch.cuda.empty_cache()
    except Exception as e:
        print("[cost] gpu skipped", e)
    return rows


if __name__ == "__main__":
    t0 = time.time()
    OUT["1_capacity"] = exp_capacity()
    OUT["2_hierarchy"] = exp_hierarchy()
    OUT["3a_time_generalization"] = exp_time_generalization()
    OUT["3b_vision"] = exp_vision()
    OUT["4_cost_1M_items"] = exp_cost()
    OUT["runtime_s"] = round(time.time() - t0, 1)
    path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "results", "hdc_probe_results.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(OUT, f, ensure_ascii=False, indent=2)
    print("saved", path, "runtime", OUT["runtime_s"], "s")
