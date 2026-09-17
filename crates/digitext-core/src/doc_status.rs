// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocStatus {
    Uploaded,
    OcrPending,
    OcrDone,
    ToSign,
    Signing,
    Signed,
    SignRejected,
    Archived,
    Disposed,
}
impl DocStatus {
    pub const ALL: [DocStatus; 9] = [
        DocStatus::Uploaded,
        DocStatus::OcrPending,
        DocStatus::OcrDone,
        DocStatus::ToSign,
        DocStatus::Signing,
        DocStatus::Signed,
        DocStatus::SignRejected,
        DocStatus::Archived,
        DocStatus::Disposed,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            DocStatus::Uploaded => "uploaded",
            DocStatus::OcrPending => "ocr_pending",
            DocStatus::OcrDone => "ocr_done",
            DocStatus::ToSign => "to_sign",
            DocStatus::Signing => "signing",
            DocStatus::Signed => "signed",
            DocStatus::SignRejected => "sign_rejected",
            DocStatus::Archived => "archived",
            DocStatus::Disposed => "disposed",
        }
    }
    pub fn is_terminal(self) -> bool {
        matches!(self, DocStatus::Disposed)
    }
    pub fn ocr_finished(self) -> bool {
        matches!(
            self,
            DocStatus::OcrDone
                | DocStatus::ToSign
                | DocStatus::Signing
                | DocStatus::Signed
                | DocStatus::SignRejected
                | DocStatus::Archived
        )
    }
    pub fn pdf2_built(self) -> bool {
        matches!(
            self,
            DocStatus::ToSign
                | DocStatus::Signing
                | DocStatus::Signed
                | DocStatus::SignRejected
                | DocStatus::Archived
        )
    }
    pub fn can_transition_to(self, r_1yd: DocStatus) -> bool {
        use DocStatus::*;
        if r_1yd == Disposed {
            return self != Disposed;
        }
        matches!(
            (self, r_1yd),
            (Uploaded, OcrPending)
                | (OcrPending, OcrDone)
                | (OcrDone, OcrPending)
                | (OcrDone, ToSign)
                | (ToSign, Signing)
                | (Signing, Signed)
                | (Signing, SignRejected)
                | (Signing, ToSign)
                | (SignRejected, ToSign)
                | (Signed, Archived)
        )
    }
}
impl fmt::Display for DocStatus {
    fn fmt(&self, r_1ye: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1ye.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("trạng thái tài liệu không hợp lệ: {0:?}")]
pub struct Tgl(pub String);
impl FromStr for DocStatus {
    type Err = Tgl;
    fn from_str(r_1yf: &str) -> Result<Self, Self::Err> {
        DocStatus::ALL
            .into_iter()
            .find(|d| d.as_str() == r_1yf)
            .ok_or_else(|| Tgl(r_1yf.to_owned()))
    }
}
impl TryFrom<String> for DocStatus {
    type Error = Tgl;
    fn try_from(r_1yg: String) -> Result<Self, Self::Error> {
        r_1yg.parse()
    }
}
#[cfg(test)]
mod tests {
    use super::DocStatus::{self, *};
    #[test]
    fn ocr_edges() {
        assert!(Uploaded.can_transition_to(OcrPending));
        assert!(OcrPending.can_transition_to(OcrDone));
        assert!(OcrDone.can_transition_to(OcrPending));
        assert!(OcrDone.can_transition_to(ToSign));
        assert!(!ToSign.can_transition_to(OcrPending));
        assert!(!Signed.can_transition_to(OcrPending));
        assert!(!OcrPending.can_transition_to(ToSign));
    }
    #[test]
    fn disposed_is_sink() {
        for s in DocStatus::ALL {
            assert_eq!(s.can_transition_to(Disposed), s != Disposed);
            assert!(!Disposed.can_transition_to(s));
        }
    }
    #[test]
    fn roundtrip_str() {
        for s in DocStatus::ALL {
            assert_eq!(s.as_str().parse::<DocStatus>().unwrap(), s);
        }
    }
}
