// Copyright (c) 2026 DigiText. All rights reserved.
use crate::live::Topic;
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
pub const EV_SESSION: &str = "upload://session";
pub const EV_FILE: &str = "upload://file";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileState {
    Uploading,
    Done,
    Skipped,
    Error,
    Cancelled,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    pub session_id: u64,
    pub id: i64,
    pub filename: String,
    pub project_id: String,
    pub ho_so_ref: Option<String>,
    pub rel_path: Option<String>,
    pub size_bytes: u64,
    pub sent_bytes: u64,
    pub state: FileState,
    pub message: Option<String>,
    pub permanent: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SessionPhase {
    Start,
    End,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEvent {
    pub session_id: u64,
    pub phase: SessionPhase,
    pub total: usize,
    pub summary: Option<serde_json::Value>,
}
pub struct Reporter {
    app: AppHandle,
    session_id: u64,
}
impl Reporter {
    pub fn new(app: AppHandle, session_id: u64) -> Self {
        Self { app, session_id }
    }
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
    pub fn session(&self, phase: SessionPhase, total: usize, summary: Option<serde_json::Value>) {
        let _ = self.app.emit(
            EV_SESSION,
            SessionEvent {
                session_id: self.session_id,
                phase,
                total,
                summary,
            },
        );
        if matches!(phase, SessionPhase::End) {
            crate::live::f_2li(
                &self.app,
                &[Topic::Collected, Topic::Docs, Topic::UploadBatches],
            );
        }
    }
    pub fn file(&self, r_1vn: &FileEvent) {
        let _ = self.app.emit(EV_FILE, r_1vn);
        if matches!(r_1vn.state, FileState::Done | FileState::Error) {
            crate::live::touch(
                &self.app,
                &[Topic::Collected, Topic::Docs, Topic::UploadBatches],
            );
        }
    }
}
pub trait Tl9 {
    fn session_id(&self) -> u64;
    fn end(&self, total: usize, summary: Option<serde_json::Value>);
}
impl Tl9 for Reporter {
    fn session_id(&self) -> u64 {
        self.session_id
    }
    fn end(&self, total: usize, summary: Option<serde_json::Value>) {
        self.session(SessionPhase::End, total, summary);
    }
}
pub struct SessionGuard<'a, S: Tl9> {
    control_end: Box<dyn Fn(u64) + Send + 'a>,
    sink: &'a S,
    closed: bool,
}
impl<'a, S: Tl9> SessionGuard<'a, S> {
    pub fn new(sink: &'a S, control_end: impl Fn(u64) + Send + 'a) -> Self {
        Self {
            control_end: Box::new(control_end),
            sink,
            closed: false,
        }
    }
    pub fn finish(mut self, total: usize, summary: serde_json::Value) {
        self.close(total, Some(summary));
    }
    fn close(&mut self, total: usize, summary: Option<serde_json::Value>) {
        if self.closed {
            return;
        }
        self.closed = true;
        (self.control_end)(self.sink.session_id());
        self.sink.end(total, summary);
    }
}
impl<S: Tl9> Drop for SessionGuard<'_, S> {
    fn drop(&mut self) {
        self.close(0, None);
    }
}
#[derive(Debug, Clone)]
pub struct FileTicket {
    session_id: u64,
    id: i64,
    filename: String,
    project_id: String,
    ho_so_ref: Option<String>,
    rel_path: Option<String>,
    size_bytes: u64,
}
impl FileTicket {
    pub fn new(
        session_id: u64,
        id: i64,
        filename: impl Into<String>,
        project_id: impl Into<String>,
        ho_so_ref: Option<String>,
        rel_path: Option<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            session_id,
            id,
            filename: filename.into(),
            project_id: project_id.into(),
            ho_so_ref,
            rel_path,
            size_bytes,
        }
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    fn event(&self, state: FileState, sent_bytes: u64, message: Option<String>) -> FileEvent {
        FileEvent {
            session_id: self.session_id,
            id: self.id,
            filename: self.filename.clone(),
            project_id: self.project_id.clone(),
            ho_so_ref: self.ho_so_ref.clone(),
            rel_path: self.rel_path.clone(),
            size_bytes: self.size_bytes,
            sent_bytes: sent_bytes.min(self.size_bytes),
            state,
            message,
            permanent: false,
        }
    }
    pub fn uploading(&self, sent_bytes: u64) -> FileEvent {
        self.event(FileState::Uploading, sent_bytes, None)
    }
    pub fn done(&self) -> FileEvent {
        self.event(FileState::Done, self.size_bytes, None)
    }
    pub fn skipped(&self) -> FileEvent {
        self.event(
            FileState::Skipped,
            self.size_bytes,
            Some(
                "TcOheSBjaOG7pyDEkcOjIGPDsyB0w6BpIGxp4buHdSBuw6B5IOKAlCBraMO0bmcgdOG6o2kgbOG6oWku"
                    .into(),
            ),
        )
    }
    pub fn cancelled(&self, sent_bytes: u64) -> FileEvent {
        self.event(
            FileState::Cancelled,
            sent_bytes,
            Some(
                "xJDDoyBo4buneSDigJQgcGjhuqduIMSRw6MgZ+G7rWkgxJHGsOG7o2MgZ2nhu68gbOG6oWksIGzhuqduIHNhdSB04bqjaSB0aeG6v3AgdOG7qyBjaOG7lyBk4burbmcu"
                    .into(),
            ),
        )
    }
    pub fn error(&self, sent_bytes: u64, message: impl Into<String>, permanent: bool) -> FileEvent {
        let mut r_1vo = self.event(FileState::Error, sent_bytes, Some(message.into()));
        r_1vo.permanent = permanent;
        r_1vo
    }
}
#[derive(Default)]
pub struct Tl8<K: Eq + std::hash::Hash> {
    state: Mutex<T32<K>>,
}
pub type ForwardControl = Tl8<i64>;
pub type CollectControl = Tl8<String>;
struct T32<K> {
    session: u64,
    cancel_all: bool,
    files: HashSet<K>,
}
impl<K: Eq + std::hash::Hash> Default for T32<K> {
    fn default() -> Self {
        Self {
            session: 0,
            cancel_all: false,
            files: HashSet::new(),
        }
    }
}
impl<K: Eq + std::hash::Hash> Tl8<K> {
    pub fn begin(&self) -> u64 {
        let mut r_1vp = self.lock();
        r_1vp.session += 1;
        r_1vp.cancel_all = false;
        r_1vp.files.clear();
        r_1vp.session
    }
    pub fn end(&self, session: u64) {
        let mut r_1vq = self.lock();
        if r_1vq.session == session {
            r_1vq.cancel_all = false;
            r_1vq.files.clear();
        }
    }
    pub fn cancel(&self, session: u64, r_1vr: Option<K>) -> bool {
        let mut r_1vs = self.lock();
        if r_1vs.session != session || r_1vs.session == 0 {
            return false;
        }
        match r_1vr {
            Some(id) => {
                r_1vs.files.insert(id);
            }
            None => r_1vs.cancel_all = true,
        }
        true
    }
    pub fn session_cancelled(&self, session: u64) -> bool {
        let r_1vt = self.lock();
        r_1vt.session == session && r_1vt.cancel_all
    }
    pub fn file_cancelled<Q>(&self, session: u64, id: &Q) -> bool
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + std::hash::Hash + ?Sized,
    {
        let r_1vu = self.lock();
        r_1vu.session == session && (r_1vu.cancel_all || r_1vu.files.contains(id))
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, T32<K>> {
        self.state.lock().unwrap_or_else(|r_1vv| r_1vv.into_inner())
    }
}
const C_CB: Duration = Duration::from_millis(150);
#[derive(Debug)]
pub struct ProgressGate {
    last_at: Option<Instant>,
    total: u64,
}
impl ProgressGate {
    pub fn new(total: u64) -> Self {
        Self {
            last_at: None,
            total,
        }
    }
    pub fn allow(&mut self, r_1vz: u64, r_1vw: Instant) -> bool {
        let r_1vx = match self.last_at {
            None => true,
            Some(r_1vy) => r_1vz >= self.total || r_1vw.duration_since(r_1vy) >= C_CB,
        };
        if r_1vx {
            self.last_at = Some(r_1vw);
        }
        r_1vx
    }
}
