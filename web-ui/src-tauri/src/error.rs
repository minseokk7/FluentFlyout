use serde::Serialize;

#[derive(Debug, Serialize, thiserror::Error)]
#[error("{message_ko}")]
pub struct AppError {
    pub code: String,
    pub message_ko: String,
    pub detail: Option<String>,
}

impl AppError {
    pub fn new(code: impl Into<String>, message_ko: impl Into<String>) -> Self {
        Self { code: code.into(), message_ko: message_ko.into(), detail: None }
    }

    pub fn with_detail(code: impl Into<String>, message_ko: impl Into<String>, detail: impl ToString) -> Self {
        Self { code: code.into(), message_ko: message_ko.into(), detail: Some(detail.to_string()) }
    }
}

pub type AppResult<T> = Result<T, AppError>;
