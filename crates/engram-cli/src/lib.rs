//! engram-cli 라이브러리 진입점.
//!
//! 통합 테스트 (`tests/parity_test.rs`) 가 CLI 명령을 외부 프로세스로 실행하지 않고
//! 동일한 Db 인스턴스 위에서 함수 호출로 검증할 수 있도록 commands / output 모듈을
//! 공개한다. main.rs 의 바이너리 엔트리는 본 lib 를 사용하지 않고 직접 commands 모듈을
//! 참조하지만, Cargo 가 동일 크레이트의 lib + bin 둘 다 빌드해도 무방하다.

pub mod commands;
pub mod output;
