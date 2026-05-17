# CLAUDE.md

이 파일은 이 저장소에서 Claude Code 가 일할 때 알아둘 컨텍스트.

## 이 저장소가 뭔지

여러 util 프로그램과 MCP 서버를 모은 **모노레포**. 패키지마다 언어/툴체인은 다르고, 루트는 공용 설정만 둔다.

- 디렉토리/네이밍 규칙: [docs/CONVENTIONS.md](docs/CONVENTIONS.md)
- 패키지 목록: [packages/README.md](packages/README.md)

## 작업할 때 자주 헷갈리는 것

- **루트에 통합 빌드 도구 없음.** workspace/turbo/nx 같은 것 안 씀. 각 패키지 안에서 해당 언어 표준 명령으로 빌드/테스트.
- **새 패키지 추가는 `packages/_template-*` 복사**. 만든 다음 [packages/README.md](packages/README.md) 표에 한 줄 추가하는 것 잊지 말 것.
- **폴더명은 kebab-case**, MCP 서버는 `-mcp-server` 접미사.

## 릴리스는 패키지별 태그로

repo 전체 버전이 없고, 패키지마다 독립 릴리스. 워크플로 파일은 `.github/workflows/release-<package>.yml`, 태그 패턴은 `<package>-v<버전>`.

예: source-backup
```bash
# Cargo.toml 의 version 을 먼저 올린 뒤
git tag source-backup-v0.2.0
git push origin source-backup-v0.2.0
```
→ GH Actions 가 Win/Linux 바이너리 빌드 후 Releases 에 첨부.

## source-backup 만의 주의점

- `Cargo.lock` 에 `image@0.25.6` 핀이 있음 (rustc 1.87 호환). `cargo update` 함부로 하면 빌드 깨질 수 있음.
- 두 개의 바이너리: `source-backup` (CLI, 기본) + `source-backup-gui` (GUI, `--features gui` 필요).
- 백업 핵심 로직은 [src/lib.rs](packages/source-backup/src/lib.rs) 의 `run_backup()`. CLI/GUI 둘 다 이걸 호출하므로 백업 동작 변경은 lib 만 손대면 됨.

## 응답 언어

사용자는 **한국어로 대화**한다. 응답도 한국어로.
