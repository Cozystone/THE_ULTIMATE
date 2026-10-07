//! Real operating-system adapter: files in a sandbox directory are objects.
//!
//! Every operation is confined to the sandbox root (checked on every path). The sandbox is
//! created under the system temp directory with a marker file and is only ever deleted if the
//! marker is present. Forbidden actions are refused and logged, never executed.
//!
//! Channels: 0 extension, 1 name id, 2 size bucket (bit length of the byte length),
//! 3 content class (FNV-1a of the bytes mod 256).
//! Actions: 1 WRITE(a) rewrites a, 2 APPEND(a), 3 PROBE(a,b) writes a fresh marker into a and
//! observes b (b changes iff a and b are the same file through a hard link),
//! 9 DELETE_OUTSIDE (tier 3, always refused).

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::{fnv1a64, Rng};
use std::fs;
use std::path::{Path, PathBuf};

pub const EXT: u16 = 0;
pub const NAME: u16 = 1;
pub const SIZE: u16 = 2;
pub const CONTENT: u16 = 3;

pub const WRITE: u16 = 1;
pub const APPEND: u16 = 2;
pub const PROBE: u16 = 3;
pub const DELETE_OUTSIDE: u16 = 9;

const MARKER: &str = ".bitmind_sandbox";

#[derive(Clone, Debug)]
pub struct FsFile {
    pub name: String,
    pub ext: i64,
    /// Hidden hard-link group (ground truth only).
    pub group: usize,
}

pub struct FsWorld {
    pub root: PathBuf,
    pub files: Vec<FsFile>,
    pub context: u64,
    pub refused: Vec<String>,
    rng: Rng,
    pub t: u64,
    pub visible: usize,
}

impl FsWorld {
    /// `n_groups` underlying files, `n_files` names, extra names are hard links to groups.
    pub fn new(seed: u64, n_files: usize, n_groups: usize, label: &str) -> std::io::Result<Self> {
        let root = std::env::temp_dir().join(format!("bitmind_fs_{label}_{seed}"));
        if root.exists() {
            if root.join(MARKER).exists() {
                fs::remove_dir_all(&root)?;
            } else {
                return Err(std::io::Error::other("refusing to reuse a directory without the sandbox marker"));
            }
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join(MARKER), b"bitmind sandbox")?;
        let mut rng = Rng::new(seed);
        let exts = ["txt", "log", "dat"];
        let mut files = Vec::new();
        for i in 0..n_files {
            let group = if i < n_groups { i } else { rng.below(n_groups as u64) as usize };
            let ext = rng.below(3) as i64;
            let name = format!("f{i:02}.{}", exts[ext as usize]);
            files.push(FsFile { name, ext, group });
        }
        let mut w = FsWorld { root, files, context: context_of(label), refused: Vec::new(), rng, t: 0, visible: 4 };
        for i in 0..n_files {
            let g = w.files[i].group;
            let p = w.path(i)?;
            if i == g {
                let n = 1 + w.rng.below(200) as usize;
                let content: Vec<u8> = (0..n).map(|_| b'a' + w.rng.below(26) as u8).collect();
                fs::write(&p, content)?;
            } else {
                let master = w.path(g)?;
                fs::hard_link(&master, &p)?;
            }
        }
        Ok(w)
    }

    fn inside(&self, p: &Path) -> bool {
        p.starts_with(&self.root) && !p.components().any(|c| matches!(c, std::path::Component::ParentDir))
    }

    fn path(&self, i: usize) -> std::io::Result<PathBuf> {
        let p = self.root.join(&self.files[i].name);
        if self.inside(&p) {
            Ok(p)
        } else {
            Err(std::io::Error::other("path escapes sandbox"))
        }
    }

    fn observe_file(&self, i: usize, slot: u16, out: &mut Vec<Token>) -> std::io::Result<()> {
        let p = self.path(i)?;
        let bytes = fs::read(&p)?;
        out.push(Token { slot, ch: EXT, val: self.files[i].ext });
        out.push(Token { slot, ch: NAME, val: i as i64 });
        out.push(Token { slot, ch: SIZE, val: 64 - (bytes.len() as u64).leading_zeros() as i64 });
        out.push(Token { slot, ch: CONTENT, val: (fnv1a64(&bytes) % 256) as i64 });
        Ok(())
    }

    /// Execute an action. Forbidden actions are refused (returns false) and logged.
    pub fn execute(&mut self, act: u16, args: &[usize]) -> std::io::Result<bool> {
        match act {
            WRITE => {
                let n = 1 + self.rng.below(300) as usize;
                let c: Vec<u8> = (0..n).map(|_| b'a' + self.rng.below(26) as u8).collect();
                fs::write(self.path(args[0])?, c)?;
            }
            APPEND => {
                use std::io::Write;
                let mut f = fs::OpenOptions::new().append(true).open(self.path(args[0])?)?;
                let n = 1 + self.rng.below(40) as usize;
                let c: Vec<u8> = (0..n).map(|_| b'a' + self.rng.below(26) as u8).collect();
                f.write_all(&c)?;
            }
            PROBE => {
                let marker = format!("probe-{}-{}", self.t, self.rng.next_u64());
                fs::write(self.path(args[0])?, marker.as_bytes())?;
            }
            _ => {
                self.refused.push(format!("t={} action {act} refused (outside permission boundary)", self.t));
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// One event. `force` selects action and file arguments; otherwise random WRITE/APPEND/PROBE.
    pub fn step(&mut self, force: Option<(u16, Vec<usize>)>) -> std::io::Result<(Event, Vec<(u16, usize)>)> {
        self.t += 1;
        let (act, args) = match force {
            Some(f) => f,
            None => {
                let a = 1 + self.rng.below(3) as u16;
                let p = self.rng.sample_distinct(self.files.len(), 2);
                if a == PROBE {
                    (a, vec![p[0], p[1]])
                } else {
                    (a, vec![p[0]])
                }
            }
        };
        let mut vis: Vec<usize> = args.clone();
        while vis.len() < self.visible.min(self.files.len()) {
            let c = self.rng.below(self.files.len() as u64) as usize;
            if !vis.contains(&c) {
                vis.push(c);
            }
        }
        let mut slots: Vec<u16> = (0..vis.len() as u16).collect();
        self.rng.shuffle(&mut slots);
        let truth: Vec<(u16, usize)> = vis.iter().enumerate().map(|(i, &f)| (slots[i], f)).collect();
        let mut pre = Vec::new();
        for &(s, f) in &truth {
            self.observe_file(f, s, &mut pre)?;
        }
        self.execute(act, &args)?;
        let mut post = Vec::new();
        for &(s, f) in &truth {
            self.observe_file(f, s, &mut post)?;
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        Ok((
            Event {
                id: 0,
                t: self.t,
                source: 20,
                context: self.context,
                pre: Scene { tokens: pre },
                act: Some(Act { id: act, args: arg_slots }),
                post: Scene { tokens: post },
                kind: Kind::Intervention,
            },
            truth,
        ))
    }

    /// Look without acting: the scene the probe `(a, b)` would start from, carrying the action, with
    /// post = pre (used by an agent to evaluate a candidate probe before choosing it).
    pub fn preview(&mut self, a: usize, b: usize) -> std::io::Result<(Event, Vec<(u16, usize)>)> {
        let mut vis: Vec<usize> = vec![a, b];
        while vis.len() < self.visible.min(self.files.len()) {
            let c = self.rng.below(self.files.len() as u64) as usize;
            if !vis.contains(&c) {
                vis.push(c);
            }
        }
        let mut slots: Vec<u16> = (0..vis.len() as u16).collect();
        self.rng.shuffle(&mut slots);
        let truth: Vec<(u16, usize)> = vis.iter().enumerate().map(|(i, &f)| (slots[i], f)).collect();
        let mut pre = Vec::new();
        for &(s, f) in &truth {
            self.observe_file(f, s, &mut pre)?;
        }
        let arg_slots = [a, b].iter().map(|x| truth.iter().find(|y| y.1 == *x).expect("arg visible").0).collect();
        Ok((
            Event {
                id: 0,
                t: self.t,
                source: 20,
                context: self.context,
                pre: Scene { tokens: pre.clone() },
                act: Some(Act { id: PROBE, args: arg_slots }),
                post: Scene { tokens: pre },
                kind: Kind::Intervention,
            },
            truth,
        ))
    }

    /// Ground truth for evaluation harnesses: the sensed content class of a file now.
    pub fn content_class(&self, i: usize) -> std::io::Result<i64> {
        Ok((fnv1a64(&fs::read(self.path(i)?)?) % 256) as i64)
    }

    /// Remove the sandbox (only if it carries the marker).
    pub fn cleanup(&self) -> std::io::Result<()> {
        if self.root.join(MARKER).exists() && self.root.starts_with(std::env::temp_dir()) {
            fs::remove_dir_all(&self.root)?;
        }
        Ok(())
    }
}

impl Drop for FsWorld {
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}
