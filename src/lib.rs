pub mod app;
pub mod backtest;
pub mod config;
pub mod core;
pub mod data;
pub mod exchange;
pub mod models;
pub mod paper;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
