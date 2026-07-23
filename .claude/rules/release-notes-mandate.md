# 📜 Release Notes Mandatory Policy (릴리즈 노트 필수 작성 수칙)

## 원칙

모든 에이전트 및 관리자는 새로운 버전 배포(`git tag v*`, Release 릴리즈)를 진행할 때 **릴리즈 노트를 반드시 검증하고 작성**해야 합니다.

1. **자동 생성 검증**: GitHub Release CI 파이프라인(`release.yml`)의 `generate_release_notes: true` 설정이 동작하여 커밋/PR/이슈 항목이 릴리즈 본문에 누락되지 않아야 합니다.
2. **에이전트 배포 전 체크**:
   - `v*` 버전 태그 생성 전, 커밋 메시지와 이슈 목록이 명확히 정리되었는지 확인합니다.
   - 사용자에게 릴리즈 노트를 브라우저로 확인할 수 있는 링크(`https://github.com/gorillaKim/engram/releases`)를 제공합니다.
3. **앱내 링크 연동**: Engram Desktop 앱 설정 페이지(`Settings.tsx`)에서 **`🌐 GitHub 릴리즈 노트 보기`** 버튼이 정상 동작해야 합니다.
