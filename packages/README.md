# packages/

각 util 프로그램 / MCP 서버는 이 디렉토리 아래 자기 폴더 하나로 존재한다.

## 패키지 목록

| 이름 | 언어 | 종류 | 설명 |
|---|---|---|---|
| [_template-python](_template-python/) | Python | template | Python 패키지 시작 템플릿 (uv 기반) |
| [_template-typescript](_template-typescript/) | TypeScript | template | TypeScript 패키지 시작 템플릿 (tsx 기반) |
| [source-backup](source-backup/) | Rust | CLI | 폴더를 zip으로 백업. `.gitignore` 우선, 폴백은 고정 블랙리스트 |

> 새 패키지를 추가하면 위 표에 한 줄 추가할 것.

## 네이밍 규칙

- 폴더명은 **kebab-case** (예: `pdf-merger`, `slack-mcp-server`).
- MCP 서버는 접미사 `-mcp-server` (예: `notion-mcp-server`).
- 템플릿은 접두사 `_template-` (목록 정렬 시 최상단으로).
