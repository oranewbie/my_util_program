# source-backup

소스 폴더를 zip으로 백업. 빌드 산출물·의존성 폴더는 자동 제외.
CLI(`source-backup`) 와 GUI(`source-backup-gui`) 두 가지로 빌드 가능.

## 동작 원리

| 상황 | 제외 기준 |
|---|---|
| 소스 폴더에 `.git/` 있음 | `.gitignore` + `.git/info/exclude` + global gitignore 적용 (ripgrep과 동일). `.git/` 폴더 자체도 제외 |
| `.git/` 없음 (또는 `--no-git`) | 고정 블랙리스트: `venv`, `.venv`, `node_modules`, `target`, `dist`, `build`, `__pycache__`, `.next`, `bin`, `obj`, `.idea`, `.vscode` 등 |

블랙리스트 전체 목록은 [src/main.rs](src/main.rs) 의 `BLACKLIST` 상수 참고.

zip 안의 파일은 원본 mtime 을 그대로 보존한다 (디렉토리 엔트리는 1980-01-01 고정 — 대부분의 zip 도구가 디렉토리 mtime은 무시함).

## 사용법

### GUI

```bash
source-backup-gui
```

창이 뜨면 **폴더 선택** → 출력 경로 확인 → 옵션(`.git 포함`, `.gitignore 무시`) 체크 → **백업 시작**. 진행률 바와 현재 파일이 실시간 표시된다.

내부적으로 dry-run 한 번 돌려 총 파일 수를 미리 구하고, 실제 백업은 백그라운드 스레드에서 수행 (UI 안 멈춤). 한글 폰트는 시스템에 설치된 Malgun Gothic(Windows) / Noto CJK(Linux) / Apple SD Gothic Neo(macOS) 를 자동으로 찾아 사용.

### CLI

```bash
# 가장 단순한 형태 — ./my-project-20260517-103000.zip 생성
source-backup ./my-project

# 출력 경로 지정
source-backup ./my-project -o /backup/snap.zip

# .git 폴더도 zip에 포함 (커밋 히스토리까지 백업)
source-backup ./my-project --include-git-dir

# .gitignore 무시하고 항상 블랙리스트로 동작
source-backup ./my-project --no-git

# zip 만들지 않고 포함될 파일 목록만 출력 (검증용)
source-backup ./my-project --dry-run
```

## 다운로드 (빌드된 바이너리)

| 위치 | 용도 |
|---|---|
| [dist/](dist/) | 로컬에서 빌드된 현재 OS 용 바이너리. **git에는 커밋되지 않음** (루트 `.gitignore` 의 `dist/` 규칙). |
| GitHub Releases | 태그 푸시 시 [.github/workflows/release-source-backup.yml](../../.github/workflows/release-source-backup.yml) 가 Windows + Linux 바이너리를 자동 업로드 |

## 빌드

### 네이티브 (현재 OS 용)

```bash
# CLI 만
cargo build --release
# 산출물: target/release/source-backup(.exe), 약 1.6MB

# CLI + GUI 둘 다 (eframe + rfd 가 끌려옴)
cargo build --release --features gui
# 산출물: 위 + target/release/source-backup-gui(.exe), 약 4.7MB
```

Linux 에서 GUI 빌드 시 시스템 의존성이 필요하다:
```bash
sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
  libxcb-xfixes0-dev libxkbcommon-dev libssl-dev
```

### 크로스 컴파일

한 머신에서 Windows·Linux 바이너리를 모두 만들 때.

**Linux/macOS → Windows:**
```bash
rustup target add x86_64-pc-windows-gnu
# Linux: sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
# 산출물: target/x86_64-pc-windows-gnu/release/source-backup.exe
```

**Windows → Linux** (가장 안정적인 방법은 WSL 안에서 그냥 네이티브 빌드):
```bash
# WSL 내부에서
cargo build --release
```

또는 [`cross`](https://github.com/cross-rs/cross) 사용 (Docker 필요):
```bash
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target x86_64-pc-windows-gnu
```

### 자동 릴리스 (GitHub Actions)

[.github/workflows/release-source-backup.yml](../../.github/workflows/release-source-backup.yml)

```bash
# Cargo.toml 의 version 을 0.2.0 으로 올렸다면:
git tag source-backup-v0.2.0
git push origin source-backup-v0.2.0
```

→ Actions 가 ubuntu/windows runner 양쪽에서 CLI + GUI 를 함께 빌드 → 총 4개 바이너리(`source-backup-*`, `source-backup-gui-*`) 가 GitHub Releases 에 첨부.

수동으로 한 번 돌려보고 싶다면 Actions 탭 → "Release source-backup" → "Run workflow" (이 경우 release 는 만들지 않고 artifact 만 업로드).

### 배포

릴리스 빌드 결과는 약 1–2MB 단일 바이너리, 런타임 의존성 없음. 그대로 복사해서 실행.

## 개발

```bash
cargo run -- ./some-folder      # 디버그 실행
cargo check                     # 타입체크만
cargo clippy                    # 린트
cargo fmt                       # 포매팅
```
