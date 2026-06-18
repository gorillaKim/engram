#!/bin/bash
set -euo pipefail

# ---------------------------------------------------------
# Engram 릴리즈 자동화 스크립트
# ---------------------------------------------------------

# 색상 정의
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Engram 릴리즈 프로세스를 시작합니다 ===${NC}\n"

# 1. 실행 환경 체크 (Git 상태 확인)
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo -e "${RED}오류: Git 리포지토리 안이 아닙니다.${NC}"
    exit 1
fi

# 현재 브랜치가 main인지 확인
CURRENT_BRANCH=$(git branch --show-current)
if [ "${CURRENT_BRANCH}" != "main" ]; then
    echo -e "${YELLOW}경고: 현재 브랜치가 main이 아닌 '${CURRENT_BRANCH}'입니다.${NC}"
    read -p "진행하시겠습니까? (y/N): " confirm_branch
    if [[ ! "${confirm_branch}" =~ ^[yY]$ ]]; then
        echo "릴리즈를 중단합니다."
        exit 1
    fi
fi

# Git 워킹 트리 상태 확인 (변경 사항이 있으면 중단 권장)
if ! git diff-index --quiet HEAD --; then
    echo -e "${YELLOW}경고: 커밋되지 않은 변경 사항이 워킹 디렉토리에 존재합니다.${NC}"
    git status -s
    read -p "무시하고 계속 진행하시겠습니까? (y/N): " confirm_dirty
    if [[ ! "${confirm_dirty}" =~ ^[yY]$ ]]; then
        echo "릴리즈를 중단합니다."
        exit 1
    fi
fi

# 2. 현재 버전 추출
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}오류: 루트 디렉토리에서 Cargo.toml을 찾을 수 없습니다.${NC}"
    exit 1
fi

CURRENT_VERSION=$(awk -F '"' '/^version =/{print $2; exit}' Cargo.toml)
if [ -z "${CURRENT_VERSION}" ]; then
    echo -e "${RED}오류: Cargo.toml에서 현재 버전을 파싱하지 못했습니다.${NC}"
    exit 1
fi

echo -e "현재 버전: ${GREEN}v${CURRENT_VERSION}${NC}"

# 다음 기본 패치 버전 계산 (예: 0.1.48 -> 0.1.49)
if [[ "${CURRENT_VERSION}" =~ ^([0-9]+)\.([0-9]+)\.([0-9]+)$ ]]; then
    MAJOR="${BASH_REMATCH[1]}"
    MINOR="${BASH_REMATCH[2]}"
    PATCH="${BASH_REMATCH[3]}"
    NEXT_PATCH_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"
else
    NEXT_PATCH_VERSION=""
fi

# 3. 신규 버전 입력 받기
if [ -n "${NEXT_PATCH_VERSION}" ]; then
    read -p "새로 배포할 버전을 입력하세요 (기본값: ${NEXT_PATCH_VERSION}): " NEW_VERSION
    NEW_VERSION="${NEW_VERSION:-${NEXT_PATCH_VERSION}}"
else
    read -p "새로 배포할 버전을 입력하세요 (예: 0.1.49): " NEW_VERSION
fi

if [[ ! "${NEW_VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}오류: 버전 형식이 유효하지 않습니다. (X.Y.Z 형태여야 합니다)${NC}"
    exit 1
fi

echo -e "설정할 신규 버전: ${GREEN}v${NEW_VERSION}${NC}"

# 4. 버전 정보 업데이트 (Cargo.toml & tauri.conf.json)
echo -e "\n${YELLOW}[1/4] 설정 파일의 버전을 업데이트하고 있습니다...${NC}"

node -e "
const fs = require('fs');

// 1. Cargo.toml 버전 업데이트
const cargoPath = 'Cargo.toml';
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, 'utf8');
  cargo = cargo.replace(/^version\s*=\s*\"[^\"]+\"/m, 'version = \"${NEW_VERSION}\"');
  fs.writeFileSync(cargoPath, cargo, 'utf8');
  console.log('✓ Cargo.toml 버전을 ${NEW_VERSION}으로 업데이트했습니다.');
} else {
  console.error('✗ Cargo.toml 파일을 찾을 수 없습니다.');
  process.exit(1);
}

// 2. tauri.conf.json 버전 업데이트
const tauriPath = 'crates/engram-desktop/src-tauri/tauri.conf.json';
if (fs.existsSync(tauriPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriPath, 'utf8'));
  tauriConf.version = '${NEW_VERSION}';
  fs.writeFileSync(tauriPath, JSON.stringify(tauriConf, null, 2), 'utf8');
  console.log('✓ tauri.conf.json 버전을 ${NEW_VERSION}으로 업데이트했습니다.');
} else {
  console.log('! tauri.conf.json 파일이 존재하지 않아 스킵합니다.');
}
"

# 5. Cargo.lock 파일 갱신 및 검사
echo -e "\n${YELLOW}[2/4] cargo check를 수행하여 Cargo.lock 버전을 동기화합니다...${NC}"
if cargo check >/dev/null 2>&1; then
    echo -e "${GREEN}✓ 의존성 및 Cargo.lock 동기화 성공${NC}"
else
    echo -e "${RED}오류: cargo check 도중 에러가 발생했습니다. 변경 사항을 확인하세요.${NC}"
    exit 1
fi

# 6. Git 스테이징 및 커밋 작성
echo -e "\n${YELLOW}[3/4] Git 버전을 커밋하고 있습니다...${NC}"
git add Cargo.toml Cargo.lock crates/engram-desktop/src-tauri/tauri.conf.json

# 변경사항이 실제로 있는지 확인
if git diff --cached --quiet; then
    echo -e "${GREEN}변경 사항이 없어 커밋 단계를 건너뜁니다.${NC}"
else
    git commit -m "chore(release): bump version to v${NEW_VERSION}"
    echo -e "${GREEN}✓ 커밋이 성공적으로 완료되었습니다.${NC}"
fi

# 7. 원격 푸시 및 태그 생성 여부 결정
echo -e "\n${YELLOW}[4/4] 깃허브 푸시 및 릴리즈 태그 생성을 시작합니다.${NC}"
read -p "GitHub에 푸시하고 v${NEW_VERSION} 태그를 배포하시겠습니까? (y/N): " confirm_push

if [[ "${confirm_push}" =~ ^[yY]$ ]]; then
    echo "원격 저장소 main 브랜치에 커밋 푸시 중..."
    git push origin main

    echo "v${NEW_VERSION} 태그 생성 및 푸시 중..."
    git tag "v${NEW_VERSION}"
    git push origin "v${NEW_VERSION}"

    echo -e "\n${GREEN}🎉 v${NEW_VERSION} 릴리즈가 성공적으로 배포되었습니다!${NC}"
    echo "GitHub Actions 워크플로우를 통해 빌드 및 자동 릴리즈 업로드가 시작됩니다."
else
    echo -e "\n${YELLOW}주의: 로컬에 변경 사항(버전 업데이트 커밋)만 유지되며 태그 배포는 보류되었습니다.${NC}"
    echo "배포하려면 수동으로 아래 명령을 실행해 주세요:"
    echo "  git push origin main"
    echo "  git tag v${NEW_VERSION}"
    echo "  git push origin v${NEW_VERSION}"
fi
