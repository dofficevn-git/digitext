// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{ser::SerializeStruct, Serialize, Serializer};
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    App { code: &'static str, message: String },
    #[error("Q8ahIHPhu58gZOG7ryBsaeG7h3UgYuG7iyBraG/DoSBo4buPbmcgKG11dGV4IHBvaXNvbmVkKS4=")]
    DbPoisoned,
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Image(#[from] image::ImageError),
}
impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        AppError::App {
            code,
            message: message.into(),
        }
    }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new("validation", message)
    }
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
    pub fn code(&self) -> &'static str {
        match self {
            AppError::App { code, .. } => code,
            AppError::DbPoisoned => "db_poisoned",
            AppError::Db(_) => "db",
            AppError::Io(_) => "io",
            AppError::Image(_) => "image",
        }
    }
}
impl From<String> for AppError {
    fn from(message: String) -> Self {
        AppError::App {
            code: "error",
            message,
        }
    }
}
impl From<&str> for AppError {
    fn from(message: &str) -> Self {
        AppError::App {
            code: "error",
            message: message.to_string(),
        }
    }
}
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, r_1vl: S) -> Result<S::Ok, S::Error> {
        let mut r_1vm = r_1vl.serialize_struct("AppError", 2)?;
        r_1vm.serialize_field("code", self.code())?;
        r_1vm.serialize_field("message", &self.to_string())?;
        r_1vm.end()
    }
}
