// Copyright (c) 2026 DigiText. All rights reserved.
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};
const C_7J: u32 = 3;
const C_7K: u32 = 20;
const C_BL: Duration = Duration::from_secs(300);
const C_5D: Duration = Duration::from_secs(900);
const MAX_ENTRIES: usize = 100_000;
#[derive(Debug, Default)]
pub struct LoginLimiter {
    users: Mutex<HashMap<String, (u32, Instant)>>,
    ips: Mutex<HashMap<IpAddr, (u32, Instant)>>,
}
fn f_13s(r_5s: u32, r_5t: u32) -> Duration {
    if r_5s < r_5t {
        return Duration::ZERO;
    }
    let r_5r = (r_5s - r_5t + 1).min(16);
    C_BL.min(Duration::from_secs(1u64 << r_5r))
}
fn gate(entry: Option<&(u32, Instant)>, r_5w: u32, r_5y: Instant) -> Result<(), u64> {
    let Some((r_5v, r_5x)) = entry else {
        return Ok(());
    };
    let r_5u = *r_5x + f_13s(*r_5v, r_5w);
    if r_5y < r_5u {
        Err((r_5u - r_5y).as_secs().max(1))
    } else {
        Ok(())
    }
}
fn f_wh(entry: &mut (u32, Instant), r_5z: Instant) {
    entry.0 = entry.0.saturating_add(1);
    entry.1 = r_5z;
}
fn prune<K: Clone + std::hash::Hash + Eq>(r_62: &mut HashMap<K, (u32, Instant)>, r_63: Instant) {
    r_62.retain(|_, (_, r_61)| r_63.duration_since(*r_61) < C_5D);
    if r_62.len() > MAX_ENTRIES {
        let mut r_60: Vec<(K, Instant)> = r_62.iter().map(|(k, (_, t))| (k.clone(), *t)).collect();
        r_60.sort_by_key(|(_, t)| *t);
        for (k, _) in r_60.into_iter().take(r_62.len() - MAX_ENTRIES) {
            r_62.remove(&k);
        }
    }
}
impl LoginLimiter {
    pub fn check(&self, r_64: &str) -> Result<(), u64> {
        self.check_at(r_64, Instant::now())
    }
    pub fn check_at(&self, r_66: &str, r_65: Instant) -> Result<(), u64> {
        let users = self.users.lock().expect("limiter lock");
        gate(users.get(r_66), C_7J, r_65)
    }
    pub fn record_failure(&self, r_67: &str) {
        self.record_failure_at(r_67, Instant::now());
    }
    pub fn record_failure_at(&self, r_69: &str, r_68: Instant) {
        let mut users = self.users.lock().expect("limiter lock");
        prune(&mut users, r_68);
        f_wh(users.entry(r_69.to_owned()).or_insert((0, r_68)), r_68);
    }
    pub fn record_success(&self, r_6a: &str) {
        self.users.lock().expect("limiter lock").remove(r_6a);
    }
    pub fn check_ip(&self, ip: IpAddr) -> Result<(), u64> {
        self.check_ip_at(ip, Instant::now())
    }
    pub fn check_ip_at(&self, ip: IpAddr, r_6b: Instant) -> Result<(), u64> {
        let ips = self.ips.lock().expect("limiter lock");
        gate(ips.get(&ip), C_7K, r_6b)
    }
    pub fn record_failure_ip(&self, ip: IpAddr) {
        self.record_failure_ip_at(ip, Instant::now());
    }
    pub fn record_failure_ip_at(&self, ip: IpAddr, r_6c: Instant) {
        let mut ips = self.ips.lock().expect("limiter lock");
        prune(&mut ips, r_6c);
        f_wh(ips.entry(ip).or_insert((0, r_6c)), r_6c);
    }
    pub fn record_success_ip(&self, ip: IpAddr) {
        self.ips.lock().expect("limiter lock").remove(&ip);
    }
}
