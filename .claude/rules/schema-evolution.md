# Rule: DB 스키마 변경 (마이그레이션)

마이그레이션은 `crates/engram-core/migrations/` 에 들어 있고 `Db::open` 안의 `sqlx::migrate!("./migrations").run(&pool)` 가 적용한다.

## 파일 명명

- 형식: `NNNN_<snake_case_name>.sql`
- 번호는 zero-pad **4자리** (`0001`, `0002`, ...)
- 이름은 변화 의도를 짧게: `0002_add_assignee.sql`, `0003_index_notes_type.sql`

## 절대 규칙

- **이미 머지된 마이그레이션 파일은 수정 금지** (`0001_init.sql` 포함). 변경하면 기존 DB 가진 사용자의 적용 이력(`_sqlx_migrations` 체크섬)과 충돌한다.
- 잘못된 마이그레이션을 발견했다면 → 새 번호의 마이그레이션으로 **수정 패치**를 추가한다.
- **예외 (좁게 적용)**: 어떤 SQLite 빌드에서도 적용에 **실패**하여 *어느 DB 의 `_sqlx_migrations` 에도 기록된 적 없는* 마이그레이션은, ① 후속 번호로 고칠 수 없고(깨진 파일이 체인을 막아 그 뒤 마이그레이션에 도달조차 못 함) ② 기록이 없으니 체크섬 충돌도 없다 → **제자리 수정이 유일하고 안전한 복구**다. 이 경우에 한해 머지된 파일을 직접 고친다 (사례: 아래 `0012` 사고).

## 안전한 변경 유형

| 변경 | 방법 |
|------|------|
| 컬럼 추가 (NULL 허용 or **상수** DEFAULT) | `ALTER TABLE x ADD COLUMN ...` — DEFAULT 는 **상수만** (⚠️ 아래 절 필독) |
| 인덱스 추가 / 제거 | `CREATE INDEX IF NOT EXISTS ...` / `DROP INDEX IF EXISTS ...` |
| CHECK 제약 추가 | SQLite 는 직접 ALTER 불가 → 신규 테이블로 옮기는 패턴 필요 (피하기) |
| NOT NULL 컬럼 추가 (기본값 없음) | **2단계 마이그레이션**: ① NULL 허용 + 백필 UPDATE → ② (다음 릴리스) NOT NULL 강제 |
| 컬럼 제거 | SQLite 3.35+ 의 `DROP COLUMN` 가능하나 신중. 인덱스 의존성 먼저 확인 |

## ⚠️ ADD COLUMN 의 DEFAULT 는 반드시 상수 (SQLite 제약)

`ALTER TABLE ... ADD COLUMN` 의 DEFAULT 에는 **비상수 표현식을 쓸 수 없다**. 쓰면 SQLite 가 거부한다: `Cannot add a column with non-constant default`.

- ❌ `ADD COLUMN updated_at TEXT NOT NULL DEFAULT (datetime('now'))`
- ❌ `... DEFAULT CURRENT_TIMESTAMP`, `... DEFAULT (<괄호 표현식>)`
- ✅ `ADD COLUMN updated_at TEXT NOT NULL DEFAULT ''` (상수)

이 제약은 **`CREATE TABLE` 에는 적용되지 않는다** — 그래서 `0001_init.sql` 의 `created_at TEXT NOT NULL DEFAULT (datetime('now'))` 는 정상이지만, **같은 구문을 ALTER 로 추가하면 실패**한다.

시간/동적 기본값 컬럼을 ALTER 로 추가하려면 **상수로 추가 후 백필**한다:

```sql
ALTER TABLE notes ADD COLUMN updated_at TEXT NOT NULL DEFAULT '';
UPDATE notes SET updated_at = created_at WHERE updated_at = '';
```

INSERT/UPDATE 시 `updated_at = datetime('now')` 를 코드에서 명시 갱신한다 (`.claude/rules/sqlx-pattern.md` 시간 컬럼 절). 최종적으로 올바른 `DEFAULT (datetime('now'))` 스키마가 필요하면 신규 테이블 재생성(`CREATE TABLE ... ; INSERT ... SELECT ; DROP ; RENAME`)으로 확정한다.

> **실제 사고 (2026-06)**: `0012_notes_updated_at.sql` 이 `ADD COLUMN ... DEFAULT (datetime('now'))` 를 사용 → v0.1.63 데스크톱 앱이 기존 DB(마이그레이션 11) 위에서 시작 시 panic("열리지 않음"). 모든 SQLite 빌드에서 거부되어 어느 DB 에도 적용된 적이 없었으므로 위 "절대 규칙 예외"에 따라 제자리 수정(상수 DEFAULT + 백필)으로 복구했다.

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

- 로컬에서 `cargo test -p engram-core` 통과 (마이그레이션은 `:memory:` 테스트에서 자동 실행됨). `release.yml` 도 태그 빌드 전 `cargo test --workspace --exclude engram-desktop` 로 게이트한다 (2026-06 추가).
- **빈 `:memory:` 만으로는 부족**: ADD COLUMN 의 비상수 DEFAULT 같은 구문 오류는 잡지만, NOT NULL 백필 누락 등 **데이터 의존 실패**는 못 잡는다. 기존 행이 있는 DB(예: 실 DB 복사본)에 대고도 한 번 적용해 본다.
- 기존 DB 가 있는 경우를 가정한 idempotent 확인: 같은 마이그레이션을 두 번 적용해도 안전한가? (`IF NOT EXISTS` 사용 권장)
