pub mod models;
pub mod repository;

pub use repository::Db;
pub use repository::retro::RetroReport;
pub use repository::session::{SessionSnapshot, EpicSnapshot, IssueSnapshot, BoardStatus, ProjectBoard, BlockedChain, IssueBoardStatus, IssueProjectBoard};
pub use repository::blocking::BlockingGraph;
pub use models::PaginatedResponse;
pub use models::apply_projection;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn normalize_nfc(s: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    s.nfc().collect()
}

pub fn normalize_nfc_opt(s: Option<String>) -> Option<String> {
    use unicode_normalization::UnicodeNormalization;
    s.map(|val| val.nfc().collect())
}
// force compile to include 0012 migration

