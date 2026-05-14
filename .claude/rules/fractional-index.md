# Rule: `tasks.ord` (Fractional Index)

`tasks.ord` 는 한 issue 안에서의 태스크 정렬 순서. **REAL** 타입, unique 제약 없음.

## 삽입 규칙

- 끝에 추가: `last_ord + 1.0` (목록이 비어 있으면 `1.0`)
- 맨 앞에 추가: `first_ord - 1.0`
- 두 태스크 사이 (`task_insert_after`): `(prev_ord + next_ord) / 2.0`

`task_create` 가 기본적으로 끝에 추가하는 동작이어야 한다. 중간 삽입은 명시적인 `task_insert_after` 만 허용.

## 정렬 / 조회

- **항상** `ORDER BY ord ASC` 로 조회. `ORDER BY id` 와 섞지 말 것.
- 같은 `ord` 값이 우연히 나오면 `ORDER BY ord ASC, id ASC` 로 안정 정렬.

## 부동소수점 한계

두 값 사이에 연속으로 평균을 끼우면 약 **52회** 후 IEEE 754 double 의 가수 정밀도가 부족해져 같은 부동소수점 값으로 수렴한다.

→ 같은 issue 안에서 정렬이 깨지면 **재정규화**가 필요.

## 재정규화

조건: 한 issue 안에서 인접 task 의 `ord` 가 동일하거나, 너무 가까워 다음 삽입이 표현 불가일 때.

```rust
let mut tx = self.pool.begin().await?;
let tasks = sqlx::query_as::<_, Task>(
    "SELECT ... FROM tasks WHERE issue_id = ? ORDER BY ord ASC, id ASC"
).bind(issue_id).fetch_all(&mut *tx).await?;

for (i, t) in tasks.iter().enumerate() {
    let new_ord = (i + 1) as f64;
    sqlx::query("UPDATE tasks SET ord = ? WHERE id = ?")
        .bind(new_ord).bind(t.id).execute(&mut *tx).await?;
}
tx.commit().await?;
```

- **반드시 트랜잭션** 안에서 일괄 처리.
- `history` 로그에는 대량 변경이라 `field = 'ord'` 일괄 기록을 생략해도 OK (행 단위 노이즈가 더 큼).
- 일반 사용자 동작이 아니므로 별도 도구로 노출하지 않는다 — 내부 maintenance 함수.

## 동시성

- WAL 모드라 reader/writer 분리되어 있지만, **재정규화 트랜잭션 도중 같은 issue 에 task 가 추가되면 충돌 가능**.
- 충돌 빈도가 낮아 현재는 `busy_timeout` 5초 재시도로 충분. 빈번해지면 issue-level lock 도입 검토.

## 테스트 시나리오

- 동일 두 ord 값 사이에 53회 삽입 후 자동 재정규화 호출 → 정렬이 1.0, 2.0, ... 으로 복구되는지.
- `task_insert_after(prev_id)` 가 `prev` 와 `next` 사이의 평균값을 만드는지.
