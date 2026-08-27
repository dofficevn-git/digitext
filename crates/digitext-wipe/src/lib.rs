// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod local;
pub mod server;
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Local,
    Server,
}
impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Local => "bcOheSBuw6B5IChkZXNrdG9wKQ==",
            Scope::Server => "bcOheSBjaOG7pw==",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Depth {
    Data,
    All,
}
impl Depth {
    pub fn as_str(self) -> &'static str {
        match self {
            Depth::Data => {
                "ZOG7ryBsaeG7h3UgbmdoaeG7h3AgduG7pSAoZ2nhu68gdMOgaSBraG/huqNuICsgZGFuaCBt4bulYyk="
            }
            Depth::All => {
                "VE/DgE4gQuG7mCAoa+G7gyBj4bqjIHTDoGkga2hv4bqjbiwgZOG7sSDDoW4sIGRhbmggbeG7pWMp"
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanItem {
    pub label: String,
    pub target: String,
    pub count: u64,
    pub bytes: Option<u64>,
}
impl PlanItem {
    pub fn new(label: impl Into<String>, target: impl Into<String>, count: u64) -> Self {
        PlanItem {
            label: label.into(),
            target: target.into(),
            count,
            bytes: None,
        }
    }
    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes = Some(bytes);
        self
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thd {
    pub scope: Scope,
    pub depth: Depth,
    pub target_id: String,
    pub items: Vec<PlanItem>,
    pub warnings: Vec<String>,
}
impl Thd {
    pub fn total(&self) -> u64 {
        self.items.iter().map(|r_5oc| r_5oc.count).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }
    pub fn expected_phrase(&self) -> String {
        match self.scope {
            Scope::Local => format!("XOA SACH MAY NAY {}", self.target_id),
            Scope::Server => {
                let sâu = match self.depth {
                    Depth::Data => "",
                    Depth::All => " TOAN BO",
                };
                format!("XOA SACH MAY CHU{sâu} {}", self.target_id)
            }
        }
    }
    pub fn phrase_matches(&self, r_5od: &str) -> bool {
        r_5od.trim() == self.expected_phrase()
    }
}
impl fmt::Display for Thd {
    fn fmt(&self, r_5of: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.scope {
            Scope::Local => writeln!(r_5of, "UGjhuqFtIHZpIDog{}", self.scope.as_str())?,
            Scope::Server => writeln!(
                r_5of,
                "UGjhuqFtIHZpIDog{} — {}",
                self.scope.as_str(),
                self.depth.as_str()
            )?,
        }
        writeln!(r_5of, "TeG7pWMgdGnDqnU6IA=={}", self.target_id)?;
        writeln!(r_5of)?;
        if self.is_empty() {
            writeln!(
                r_5of,
                "ICAoa2jDtG5nIHTDrG0gdGjhuqV5IGThu68gbGnhu4d1IG7DoG8gxJHhu4MgeG/DoSk="
            )?;
        } else {
            for r_5og in &self.items {
                let r_5oe = match r_5og.bytes {
                    Some(b) => format!(" · {}", f_1hp(b)),
                    None => String::new(),
                };
                writeln!(
                    r_5of,
                    "  {:<34} {:>9}{}\n      {}",
                    r_5og.label, r_5og.count, r_5oe, r_5og.target
                )?;
            }
            writeln!(r_5of, "CiAgVOG7lE5HOiA={}IG3hu6Vj", self.total())?;
        }
        for w in &self.warnings {
            writeln!(r_5of, "\n  ⚠ {w}")?;
        }
        Ok(())
    }
}
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub removed: u64,
    pub errors: Vec<String>,
}
impl Report {
    pub fn merge(&mut self, r_5oh: Report) {
        self.removed += r_5oh.removed;
        self.errors.extend(r_5oh.errors);
    }
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}
impl fmt::Display for Report {
    fn fmt(&self, r_5oi: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(r_5oi, "xJDDoyB4b8OhIA=={}IG3hu6Vj", self.removed)?;
        if self.errors.is_empty() {
            write!(r_5oi, "IMK3IGtow7RuZyBs4buXaQ==")
        } else {
            write!(r_5oi, " · {}IEzhu5ZJOg==", self.errors.len())?;
            for e in &self.errors {
                write!(r_5oi, "\n    - {e}")?;
            }
            Ok(())
        }
    }
}
pub fn f_1hp(b: u64) -> String {
    const K: u64 = 1024;
    match b {
        0..=1023 => format!("{b} B"),
        _ if b < K * K => format!("{:.1} KiB", b as f64 / K as f64),
        _ if b < K * K * K => format!("{:.1} MiB", b as f64 / (K * K) as f64),
        _ => format!("{:.2} GiB", b as f64 / (K * K * K) as f64),
    }
}
