# Rule: sqlx 0.7 사용 패턴

`engram-core` 의 모든 DB 접근은 다음 규칙을 따른다.

## 매크로 vs 런타임 함수

- **`sqlx::query!` / `sqlx::query_as!` 매크로 사용 금지**.
  컴파일 타임 검증은 offline mode (`sqlx-data.json`) 셋업·CI 동기화 부담을 만든다.
- 대신 런타임 함수를 사용:
  - `sqlx::query("...")` — 변경 쿼리 (INSERT/UPDATE/DELETE)
  - `sqlx::query_scalar::<_, T>("...")` — 단일 컬럼/스칼라
  - `sqlx::query_as::<_, T>("...")` — `T: sqlx::FromRow` 로의 매핑

## 파라미터 바인딩

- `?` 플레이스홀더 + `.bind(value)` 사슬. 문자열 보간(`format!`) 금지.
- 동적 SQL이 필요할 때만 `String` 으로 SQL을 조립하고, **값은 항상 bind**:

  ```rust
  let mut sql = "SELECT ... WHERE 1=1".to_string();
  if sprint_id.is_some()   { sql.push_str(" AND sprint_id = ?"); }
  if project_key.is_some() { sql.push_str(" AND project_key = ?"); }

  let mut q = sqlx::query_as::<_, Epic>(&sql);
  if let Some(s) = sprint_id   { q = q.bind(s); }
  if let Some(p) = project_key { q = q.bind(p); }
  q.fetch_all(&self.pool).await?
  ```

  → `epic_list` 구현이 참고용 표준 (`crates/engram-core/src/repository/epic.rs`).

## 단건 조회 — `fetch_one` vs `fetch_optional`

- `fetch_one`: 결과가 없으면 `sqlx::Error::RowNotFound` 가 곧장 `Error::Db` 로 매핑된다 → 사용자에게 "DB 에러"로 노출됨.
- **NotFound 의도라면 항상 `fetch_optional` + `ok_or_else(|| Error::NotFound(...))`**:

  ```rust
  pub async fn epic_get(&self, id: i64) -> Result<Epic> {
      sqlx::query_as::<_, Epic>("SELECT ... FROM epics WHERE id = ?")
          .bind(id)
          .fetch_optional(&self.pool)
          .await?
          .ok_or_else(|| Error::NotFound(format!("epic:{id}")))
  }
  ```

## INSERT 후 ID 회수

- SQLite 3.35+ `RETURNING` 절 사용:

  ```rust
  let id = sqlx::query_scalar::<_, i64>(
      "INSERT INTO epics (...) VALUES (?, ?, ?, ?) RETURNING id",
  )
  .bind(...)
  .fetch_one(&self.pool)
  .await?;
  self.epic_get(id).await
  ```

- `last_insert_rowid()` 는 동시성 안전성이 떨어지므로 사용 금지.

## 트랜잭션

- 2개 이상의 쓰기가 한 단위여야 할 때 (`issue_link` 생성 + `history` 기록 등):

  ```rust
  let mut tx = self.pool.begin().await?;
  sqlx::query("INSERT ...").execute(&mut *tx).await?;
  sqlx::query("INSERT INTO history ...").execute(&mut *tx).await?;
  tx.commit().await?;
  ```

- 한 건 쓰기라면 풀에 직접 실행 (불필요한 트랜잭션 금지).

## Enum ↔ TEXT 컬럼

- 일관된 매핑 패턴 (모든 상태 enum 에 동일하게):

  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
  #[sqlx(type_name = "TEXT")]
  #[serde(rename_all = "snake_case")]
  pub enum IssueStatus { Draft, Approved, Todo, InProgress, /* ... */ }
  ```

- 동적 SQL 에서 enum 을 bind 할 때는 `serde_json::to_value(&status).unwrap().as_str().unwrap().to_string()` 패턴 (현재 `epic_update` 가 사용).

## 시간 컬럼

- 모든 `created_at` / `updated_at` 은 `TEXT NOT NULL DEFAULT (datetime('now'))`.
- UPDATE 시 `updated_at = datetime('now')` 를 **함께 갱신**한다 (DB 트리거 없음).

## PRAGMA / 연결

- `Db::open` 한 곳에서만 풀을 만든다. PRAGMA(WAL, busy_timeout, foreign_keys)는 마이그레이션 첫 파일이 적용한다 — 변경하지 말 것.
