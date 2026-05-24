pub mod blocking;
pub mod epic;
pub mod history;
pub mod issue;
pub mod mission;
pub mod note;
pub mod retro;
pub mod session;
pub mod sprint;
pub mod task;
pub mod task_test;

use sqlx::SqlitePool;

/// 단일 중앙 DB 연결 래퍼 (~/.engram/engram.db)
#[derive(Clone)]
pub struct Db {
    pub(crate) pool: SqlitePool,
}

impl Db {
    pub async fn open(path: &str) -> crate::Result<Self> {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_millis(5000))
            .foreign_keys(true);

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// 기본 DB 경로: ~/.engram/engram.db
    pub async fn open_default() -> crate::Result<Self> {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let dir = format!("{home}/.engram");
        std::fs::create_dir_all(&dir).ok();
        Self::open(&format!("{dir}/engram.db")).await
    }

    /// 테스트용 인메모리 DB — WAL 없이 migration만 실행
    pub async fn open_in_memory() -> crate::Result<Self> {
      use sqlx::sqlite::SqliteConnectOptions;
      use std::str::FromStr;

      let options = SqliteConnectOptions::from_str("sqlite::memory:")
          .map_err(crate::Error::Db)?
          .foreign_keys(true);

      let pool = sqlx::sqlite::SqlitePoolOptions::new()
          .max_connections(5)
          .connect_with(options)
          .await?;
      sqlx::migrate!("./migrations").run(&pool).await?;

      Ok(Self { pool })
    }

    /// 내부 커넥션 풀을 반환 (주로 통합 테스트용)
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
