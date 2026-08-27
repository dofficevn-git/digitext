// Copyright (c) 2026 DigiText. All rights reserved.
use crate::live::Topic;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
pub const EV_SESSION: &str = "collect://session";
pub const C_6N: &str = "collect://stage";
pub const EV_FILE: &str = "collect://file";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    Lan,
    Disk,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileState {
    Downloading,
    Done,
    Skipped,
    Error,
    Cancelled,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEvent {
    pub session_id: u64,
    pub key: String,
    pub filename: String,
    pub source_username: Option<String>,
    pub peer: String,
    pub ho_so_ref: Option<String>,
    pub size_bytes: u64,
    pub received_bytes: u64,
    pub state: FileState,
    pub message: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Local,
    Kho,
    Discover,
    Peer,
    Disk,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlx {
    pub session_id: u64,
    pub stage: Stage,
    pub label: String,
    pub peers: usize,
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
    pub kind: Kind,
    pub summary: Option<serde_json::Value>,
}
pub struct Reporter {
    app: AppHandle,
    session_id: u64,
    kind: Kind,
}
impl Reporter {
    pub fn new(app: AppHandle, session_id: u64, kind: Kind) -> Self {
        Self {
            app,
            session_id,
            kind,
        }
    }
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
    pub fn session(&self, phase: SessionPhase, summary: Option<serde_json::Value>) {
        let _ = self.app.emit(
            EV_SESSION,
            SessionEvent {
                session_id: self.session_id,
                phase,
                kind: self.kind,
                summary,
            },
        );
        if matches!(phase, SessionPhase::End) {
            crate::live::f_2li(&self.app, &[Topic::Collected, Topic::Hoso]);
        }
    }
    pub fn stage(&self, stage: Stage, label: impl Into<String>, peers: usize) {
        let _ = self.app.emit(
            C_6N,
            Tlx {
                session_id: self.session_id,
                stage,
                label: label.into(),
                peers,
            },
        );
    }
    pub fn file(&self, r_15i: &FileEvent) {
        let _ = self.app.emit(EV_FILE, r_15i);
        if matches!(r_15i.state, FileState::Done | FileState::Error) {
            crate::live::touch(&self.app, &[Topic::Collected, Topic::Hoso]);
        }
    }
}
impl crate::forward_progress::Tl9 for Reporter {
    fn session_id(&self) -> u64 {
        self.session_id
    }
    fn end(&self, _total: usize, summary: Option<serde_json::Value>) {
        self.session(SessionPhase::End, summary);
    }
}
#[derive(Debug, Clone)]
pub struct FileTicket {
    session_id: u64,
    key: String,
    filename: String,
    source_username: Option<String>,
    peer: String,
    ho_so_ref: Option<String>,
    size_bytes: u64,
}
impl FileTicket {
    pub fn new(
        session_id: u64,
        key: impl Into<String>,
        filename: impl Into<String>,
        source_username: Option<String>,
        peer: impl Into<String>,
        ho_so_ref: Option<String>,
        size_bytes: u64,
    ) -> Self {
        Self {
            session_id,
            key: key.into(),
            filename: filename.into(),
            source_username,
            peer: peer.into(),
            ho_so_ref,
            size_bytes,
        }
    }
    pub fn disk(
        session_id: u64,
        index: usize,
        filename: impl Into<String>,
        r_15j: Option<String>,
        size_bytes: u64,
    ) -> Self {
        Self::new(
            session_id,
            format!("disk-{index}"),
            filename,
            None,
            "",
            r_15j,
            size_bytes,
        )
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    fn event(&self, state: FileState, r_15k: u64, message: Option<String>) -> FileEvent {
        FileEvent {
            session_id: self.session_id,
            key: self.key.clone(),
            filename: self.filename.clone(),
            source_username: self.source_username.clone(),
            peer: self.peer.clone(),
            ho_so_ref: self.ho_so_ref.clone(),
            size_bytes: self.size_bytes,
            received_bytes: r_15k.min(self.size_bytes),
            state,
            message,
        }
    }
    pub fn downloading(&self, r_15l: u64) -> FileEvent {
        self.event(FileState::Downloading, r_15l, None)
    }
    pub fn done(&self) -> FileEvent {
        self.event(FileState::Done, self.size_bytes, None)
    }
    pub fn skipped(&self, r_15m: impl Into<String>) -> FileEvent {
        self.event(FileState::Skipped, self.size_bytes, Some(r_15m.into()))
    }
    pub fn cancelled(&self, r_15n: u64) -> FileEvent {
        self.event(
            FileState::Cancelled,
            r_15n,
            Some(
                "Đã hủy — phần đã tải được giữ lại, lần gom sau tải tiếp."
                    .into(),
            ),
        )
    }
    pub fn error(&self, r_15o: u64, message: impl Into<String>) -> FileEvent {
        self.event(FileState::Error, r_15o, Some(message.into()))
    }
}
