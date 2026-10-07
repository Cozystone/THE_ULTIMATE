//! Phase G held-out worlds. Written after the BITMIND v0.1 freeze; none of them was seen while the
//! learner was developed, and the learner may not be changed to pass them.
//!
//! * `DirWorld` (real OS): files live in hidden directories. RELOCATE(a, b) renames the directory
//!   that contains `a` and observes `b`; b's sensed location changes iff a and b share a directory.
//!   Same pair-equivalence structure as hard links, different real phenomenon.
//! * `RoWorld` (real OS, exploratory): the read-only attribute is a hidden, history-dependent state.
//!   WRITE(a) changes a's content iff a is writable; LOCK(a) / UNLOCK(a) set / clear the attribute,
//!   which is never sensed.
//! * `LockWorld` (microworld, direct episodes): a key opens a lock iff their colours are equal AND
//!   the key is smaller than the lock: a conjunction of two relations on two channels. Test values
//!   (colours, sizes) are disjoint from training values.
//! * `MagnetWorld` (microworld, grounded events): tokens carry a hidden polarity; PROBE(a, b) flips
//!   b iff the polarities differ (anti-equivalence latent cause).
//!
//! Every real-OS operation is confined to a sandbox under the system temp directory that carries a
//! marker file; the sandbox is removed only if the marker is present.

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::{context_of, Entity, Episode, Kind as EpKind};
use hdc_core::rng::{fnv1a64, Rng};
use std::fs;
use std::path::{Path, PathBuf};

const MARKER: &str = ".bitmind_sandbox";

// sensed channels shared by the real-OS worlds
pub const EXT: u16 = 0;
pub const NAME: u16 = 1;
pub const SIZE: u16 = 2;
/// DirWorld: location class (FNV-1a of the full path mod 256). RoWorld: content class.
pub const LOC: u16 = 3;
pub const CONTENT: u16 = 3;

/// D049: ordinal sensor channels of the real-OS held-out worlds (size bucket).
pub const OS_ORDINAL: &[u16] = &[SIZE];
/// D049: LockWorld size is a magnitude; colour, shape and id are nominal.
pub const LOCK_ORDINAL: &[u16] = &[SIZE_CH];
/// D049: no MagnetWorld channel is a magnitude.
pub const MAGNET_ORDINAL: &[u16] = &[];

pub const RELOCATE: u16 = 4;
pub const WRITE: u16 = 1;
pub const LOCK: u16 = 5;
pub const UNLOCK: u16 = 6;

fn sandbox(label: &str, seed: u64) -> std::io::Result<PathBuf> {
    let root = std::env::temp_dir().join(format!("bitmind_g_{label}_{seed}"));
    if root.exists() {
        if root.join(MARKER).exists() {
            clear_readonly(&root);
            fs::remove_dir_all(&root)?;
        } else {
            return Err(std::io::Error::other("refusing to reuse a directory without the sandbox marker"));
        }
    }
    fs::create_dir_all(&root)?;
    fs::write(root.join(MARKER), b"bitmind sandbox")?;
    Ok(root)
}

fn clear_readonly(dir: &Path) {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                clear_readonly(&p);
            } else if let Ok(m) = fs::metadata(&p) {
                let mut perm = m.permissions();
                if perm.readonly() {
                    #[allow(clippy::permissions_set_readonly_false)]
                    perm.set_readonly(false);
                    let _ = fs::set_permissions(&p, perm);
                }
            }
        }
    }
}

fn remove_sandbox(root: &Path) {
    if root.join(MARKER).exists() && root.starts_with(std::env::temp_dir()) {
        clear_readonly(root);
        let _ = fs::remove_dir_all(root);
    }
}

fn inside(root: &Path, p: &Path) -> bool {
    p.starts_with(root) && !p.components().any(|c| matches!(c, std::path::Component::ParentDir))
}

/// Shared event construction: `vis` objects, shuffled slots, pre/post observation around `act`.
fn make_event(t: u64, source: u32, context: u64, act: u16, args: &[usize], truth: &[(u16, usize)], pre: Vec<Token>, post: Vec<Token>) -> Event {
    let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
    Event {
        id: 0,
        t,
        source,
        context,
        pre: Scene { tokens: pre },
        act: Some(Act { id: act, args: arg_slots }),
        post: Scene { tokens: post },
        kind: Kind::Intervention,
    }
}

fn pick_visible(rng: &mut Rng, args: &[usize], n: usize, visible: usize) -> Vec<(u16, usize)> {
    let mut vis: Vec<usize> = args.to_vec();
    while vis.len() < visible.min(n) {
        let c = rng.below(n as u64) as usize;
        if !vis.contains(&c) {
            vis.push(c);
        }
    }
    let mut slots: Vec<u16> = (0..vis.len() as u16).collect();
    rng.shuffle(&mut slots);
    vis.iter().enumerate().map(|(i, &f)| (slots[i], f)).collect()
}

// ------------------------------------------------------------------------------------ DirWorld
pub struct DirWorld {
    pub root: PathBuf,
    /// file name and hidden directory index (ground truth only)
    pub files: Vec<(String, i64, usize)>,
    /// current directory names
    pub dirs: Vec<String>,
    pub context: u64,
    rng: Rng,
    pub t: u64,
    pub visible: usize,
}

impl DirWorld {
    pub fn new(seed: u64, n_files: usize, n_dirs: usize, label: &str) -> std::io::Result<Self> {
        let root = sandbox(label, seed)?;
        let mut rng = Rng::new(seed);
        let dirs: Vec<String> = (0..n_dirs).map(|d| format!("d{d}_{}", rng.next_u64() % 100000)).collect();
        for d in &dirs {
            fs::create_dir_all(root.join(d))?;
        }
        let exts = ["txt", "log", "dat"];
        let mut files = Vec::new();
        for i in 0..n_files {
            let dir = if i < n_dirs { i } else { rng.below(n_dirs as u64) as usize };
            let ext = rng.below(3) as i64;
            files.push((format!("f{i:02}.{}", exts[ext as usize]), ext, dir));
        }
        let w = DirWorld { root, files, dirs, context: context_of(label), rng, t: 0, visible: 4 };
        for i in 0..n_files {
            let p = w.path(i)?;
            let n = 1 + (fnv1a64(p.to_string_lossy().as_bytes()) % 300) as usize;
            fs::write(&p, vec![b'x'; n])?;
        }
        Ok(w)
    }

    fn path(&self, i: usize) -> std::io::Result<PathBuf> {
        let (name, _, d) = &self.files[i];
        let p = self.root.join(&self.dirs[*d]).join(name);
        if inside(&self.root, &p) {
            Ok(p)
        } else {
            Err(std::io::Error::other("path escapes sandbox"))
        }
    }

    fn observe(&self, i: usize, slot: u16, out: &mut Vec<Token>) -> std::io::Result<()> {
        let p = self.path(i)?;
        let len = fs::metadata(&p)?.len();
        out.push(Token { slot, ch: EXT, val: self.files[i].1 });
        out.push(Token { slot, ch: NAME, val: i as i64 });
        out.push(Token { slot, ch: SIZE, val: 64 - len.leading_zeros() as i64 });
        // the agent senses where the file is (an opaque location class), not which folder holds it
        let rel = p.strip_prefix(&self.root).unwrap_or(&p).to_string_lossy().to_string();
        out.push(Token { slot, ch: LOC, val: (fnv1a64(rel.as_bytes()) % 256) as i64 });
        Ok(())
    }

    /// RELOCATE(a, b): the operating system renames the folder that contains `a`.
    fn relocate(&mut self, a: usize) -> std::io::Result<()> {
        let d = self.files[a].2;
        let new_name = format!("d{d}_{}", self.rng.next_u64() % 100000);
        let (from, to) = (self.root.join(&self.dirs[d]), self.root.join(&new_name));
        if !inside(&self.root, &from) || !inside(&self.root, &to) || to.exists() {
            return Ok(());
        }
        fs::rename(from, to)?;
        self.dirs[d] = new_name;
        Ok(())
    }

    pub fn step(&mut self, a: usize, b: usize) -> std::io::Result<(Event, Vec<(u16, usize)>)> {
        self.t += 1;
        let truth = pick_visible(&mut self.rng, &[a, b], self.files.len(), self.visible);
        let mut pre = Vec::new();
        for &(s, f) in &truth {
            self.observe(f, s, &mut pre)?;
        }
        self.relocate(a)?;
        let mut post = Vec::new();
        for &(s, f) in &truth {
            self.observe(f, s, &mut post)?;
        }
        Ok((make_event(self.t, 21, self.context, RELOCATE, &[a, b], &truth, pre, post), truth))
    }

    pub fn same_dir(&self, a: usize, b: usize) -> bool {
        self.files[a].2 == self.files[b].2
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}

impl Drop for DirWorld {
    fn drop(&mut self) {
        remove_sandbox(&self.root);
    }
}

// ------------------------------------------------------------------------------------- RoWorld
pub struct RoWorld {
    pub root: PathBuf,
    pub names: Vec<(String, i64)>,
    pub context: u64,
    rng: Rng,
    pub t: u64,
    pub visible: usize,
}

impl RoWorld {
    pub fn new(seed: u64, n_files: usize, label: &str) -> std::io::Result<Self> {
        let root = sandbox(label, seed)?;
        let mut rng = Rng::new(seed);
        let exts = ["txt", "log", "dat"];
        let names: Vec<(String, i64)> = (0..n_files)
            .map(|i| {
                let e = rng.below(3) as i64;
                (format!("r{i:02}.{}", exts[e as usize]), e)
            })
            .collect();
        let w = RoWorld { root, names, context: context_of(label), rng, t: 0, visible: 3 };
        for i in 0..n_files {
            fs::write(w.path(i)?, b"initial")?;
        }
        Ok(w)
    }

    fn path(&self, i: usize) -> std::io::Result<PathBuf> {
        let p = self.root.join(&self.names[i].0);
        if inside(&self.root, &p) {
            Ok(p)
        } else {
            Err(std::io::Error::other("path escapes sandbox"))
        }
    }

    /// Ground truth: whether the file is currently read-only.
    pub fn locked(&self, i: usize) -> bool {
        self.path(i).ok().and_then(|p| fs::metadata(p).ok()).map(|m| m.permissions().readonly()).unwrap_or(false)
    }

    fn observe(&self, i: usize, slot: u16, out: &mut Vec<Token>) -> std::io::Result<()> {
        let bytes = fs::read(self.path(i)?)?;
        out.push(Token { slot, ch: EXT, val: self.names[i].1 });
        out.push(Token { slot, ch: NAME, val: i as i64 });
        out.push(Token { slot, ch: SIZE, val: 64 - (bytes.len() as u64).leading_zeros() as i64 });
        out.push(Token { slot, ch: CONTENT, val: (fnv1a64(&bytes) % 256) as i64 });
        Ok(())
    }

    fn execute(&mut self, act: u16, a: usize) -> std::io::Result<()> {
        let p = self.path(a)?;
        match act {
            WRITE => {
                let n = 1 + self.rng.below(200) as usize;
                let c: Vec<u8> = (0..n).map(|_| b'a' + self.rng.below(26) as u8).collect();
                // a write to a read-only file fails: the world simply does not change
                let _ = fs::write(&p, c);
            }
            LOCK | UNLOCK => {
                let mut perm = fs::metadata(&p)?.permissions();
                #[allow(clippy::permissions_set_readonly_false)]
                perm.set_readonly(act == LOCK);
                fs::set_permissions(&p, perm)?;
            }
            _ => {}
        }
        Ok(())
    }

    pub fn step(&mut self, act: u16, a: usize) -> std::io::Result<(Event, Vec<(u16, usize)>)> {
        self.t += 1;
        let truth = pick_visible(&mut self.rng, &[a], self.names.len(), self.visible);
        let mut pre = Vec::new();
        for &(s, f) in &truth {
            self.observe(f, s, &mut pre)?;
        }
        self.execute(act, a)?;
        let mut post = Vec::new();
        for &(s, f) in &truth {
            self.observe(f, s, &mut post)?;
        }
        Ok((make_event(self.t, 22, self.context, act, &[a], &truth, pre, post), truth))
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}

impl Drop for RoWorld {
    fn drop(&mut self) {
        remove_sandbox(&self.root);
    }
}

// ----------------------------------------------------------------------------------- LockWorld
pub const COLOUR: u16 = 0;
pub const SIZE_CH: u16 = 1;
pub const SHAPE: u16 = 2;
pub const ID: u16 = 3;
pub const OPEN: u32 = 0;

#[derive(Clone, Copy, Debug)]
pub struct Thing {
    pub colour: i64,
    pub size: i64,
    pub shape: i64,
    pub id: i64,
}

pub struct LockWorld {
    pub context: u64,
    rng: Rng,
    next_id: i64,
    t: u64,
}

impl LockWorld {
    pub fn new(seed: u64, label: &str) -> Self {
        LockWorld { context: context_of(label), rng: Rng::new(seed), next_id: 0, t: 0 }
    }

    /// Ground truth: the key opens the lock iff same colour and key smaller than lock.
    pub fn opens(k: &Thing, l: &Thing) -> bool {
        k.colour == l.colour && k.size < l.size
    }

    /// A key-lock pair from the training range (colours 0..6, sizes 1..=10) or the test range
    /// (colours 6..10, sizes 11..=20). Equal colours half of the time.
    pub fn pair(&mut self, test: bool) -> (Thing, Thing) {
        let (c0, cn, s0, sn) = if test { (6, 4, 11, 10) } else { (0, 6, 1, 10) };
        let ck = c0 + self.rng.below(cn) as i64;
        let cl = if self.rng.below(2) == 0 { ck } else { c0 + self.rng.below(cn) as i64 };
        let mut thing = |c: i64, rng: &mut Rng| {
            self.next_id += 1;
            Thing { colour: c, size: s0 + rng.below(sn) as i64, shape: rng.below(4) as i64, id: self.next_id }
        };
        let k = thing(ck, &mut self.rng);
        let l = thing(cl, &mut self.rng);
        (k, l)
    }

    pub fn episode(&mut self, k: &Thing, l: &Thing) -> Episode {
        self.t += 1;
        let ent = |x: &Thing| Entity::bound(x.id, &[(COLOUR, x.colour), (SIZE_CH, x.size), (SHAPE, x.shape), (ID, x.id)]);
        Episode {
            id: 0,
            t: self.t,
            context: self.context,
            source: 0,
            action: 1,
            roles: vec![ent(k), ent(l)],
            n_args: 2,
            outcomes: vec![(OPEN, Self::opens(k, l) as i64)],
            kind: EpKind::Intervention,
        }
    }
}

// --------------------------------------------------------------------------------- MagnetWorld
pub const KIND: u16 = 0;
pub const MCOLOUR: u16 = 1;
pub const MARK: u16 = 2;
pub const ON: u16 = 3;
pub const PROBE: u16 = 3;

pub struct MagnetWorld {
    /// (colour, mark, on, hidden polarity)
    pub toks: Vec<(i64, i64, i64, i64)>,
    pub context: u64,
    rng: Rng,
    pub t: u64,
    pub visible: usize,
}

impl MagnetWorld {
    pub fn new(seed: u64, n: usize, label: &str) -> Self {
        let mut rng = Rng::new(seed);
        // polarity: a random balanced assignment, unrelated to the mark (an index-parity
        // assignment leaked the hidden cause into mark differences; found in the smoke test)
        let mut pol: Vec<i64> = (0..n).map(|i| (i % 2) as i64).collect();
        rng.shuffle(&mut pol);
        let toks = (0..n).map(|i| (rng.below(4) as i64, 300 + i as i64, 0, pol[i])).collect();
        MagnetWorld { toks, context: context_of(label), rng, t: 0, visible: 4 }
    }

    pub fn differ(&self, a: usize, b: usize) -> bool {
        self.toks[a].3 != self.toks[b].3
    }

    fn emit(&self, i: usize, slot: u16, out: &mut Vec<Token>) {
        let (c, m, on, _) = self.toks[i];
        for (ch, v) in [(KIND, 70), (MCOLOUR, c), (MARK, m), (ON, on)] {
            out.push(Token { slot, ch, val: v });
        }
    }

    pub fn step(&mut self, a: usize, b: usize) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let truth = pick_visible(&mut self.rng, &[a, b], self.toks.len(), self.visible);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        if self.differ(a, b) {
            self.toks[b].2 ^= 1;
        }
        let mut post = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut post);
        }
        (make_event(self.t, 31, self.context, PROBE, &[a, b], &truth, pre, post), truth)
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}
