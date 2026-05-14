# Rule: DB 스키마 변경 (마이그레이션)

마이그레이션은 `crates/engram-core/migrations/` 에 들어 있고 `Db::open` 안의 `sqlx::migrate!("./migrations").run(&pool)` 가 적용한다.

## 파일 명명

- 형식: `NNNN_<snake_case_name>.sql`
- 번호는 zero-pad **4자리** (`0001`, `0002`, ...)
- 이름은 변화 의도를 짧게: `0002_add_assignee.sql`, `0003_index_notes_type.sql`

## 절대 규칙

- **이미 머지된 마이그레이션 파일은 수정 금지** (`0001_init.sql` 포함). 변경하면 기존 DB 가진 사용자의 적용 이력과 충돌한다.
- 잘못된 마이그레이션을 발견했다면 → 새 번호의 마이그레이션으로 **수정 패치**를 추가한다.

## 안전한 변경 유형

| 변경 | 방법 |
|------|------|
| 컬럼 추가 (NULL 허용 or DEFAULT) | `ALTER TABLE x ADD COLUMN ...` |
| 인덱스 추가 / 제거 | `CREATE INDEX IF NOT EXISTS ...` / `DROP INDEX IF EXISTS ...` |
| CHECK 제약 추가 | SQLite 는 직접 ALTER 불가 → 신규 테이블로 옮기는 패턴 필요 (피하기) |
| NOT NULL 컬럼 추가 (기본값 없음) | **2단계 마이그레이션**: ① NULL 허용 + 백필 UPDATE → ② (다음 릴리스) NOT NULL 강제 |
| 컬럼 제거 | SQLite 3.35+ 의 `DROP COLUMN` 가능하나 신중. 인덱스 의존성 먼저 확인 |

## 권장 헤더 (모든 마이그레이션 첫 줄)

```sql
-- migrations/NNNN_<name>.sql
-- Purpose: <한 줄 설명>
```

`PRAGMA foreign_keys` 같은 세션 PRAGMA 를 마이그레이션 안에서 끄지 말 것 — `0001_init.sql` 이 켰다.

## Repository / 모델 동기화

마이그레이션을 추가하면 보통 다음을 같이 수정한다:
- `crates/engram-core/src/models/<entity>.rs` — 필드 추가 / enum variant 추가
- `crates/engram-core/src/repository/<entity>.rs` — `SELECT` 컬럼 목록, `INSERT`, `UPDATE` 갱신
- 새 enum variant 면 `CHECK` 제약과 Rust enum 을 **둘 다 갱신**

## 검증

- 로컬에서 `cargo test -p engram-core` 통과 (마이그레이션은 `:memory:` 테스트에서 자동 실행됨)
- 기존 DB 가 있는 경우를 가정한 idempotent 확인: 같은 마이그레이션을 두 번 적용해도 안전한가? (`IF NOT EXISTS` 사용 권장)
