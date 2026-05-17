# 모노레포 규칙

## 폴더 레이아웃

```
packages/<package-name>/
├── README.md          # 필수. 무엇이고 어떻게 실행/테스트하는지
├── src/               # 소스코드
└── tests/             # 테스트
```

## 패키지 네이밍

- 폴더명은 **kebab-case**: `pdf-merger`, `markdown-linter`.
- 역할로 묶기보다 **무엇을 하는지**가 드러나게.
- MCP 서버는 `-mcp-server` 접미사: `slack-mcp-server`, `notion-mcp-server`.
- 템플릿/스캐폴드는 `_template-` 접두사 (목록 최상단).

## 언어 선택

| 용도 | 권장 |
|---|---|
| MCP 서버 | TypeScript (공식 SDK가 가장 성숙). Python 도 가능 |
| 파일/텍스트 처리 util | Python |
| 단일 바이너리 + 크로스플랫폼 배포 | Rust 또는 Go |
| Windows 통합/시스템 util | C# / .NET |

확신이 없으면 **Python**으로 시작 — 가장 가벼움.

## 새 패키지 만들기

### Python

```bash
cp -r packages/_template-python packages/my-tool
cd packages/my-tool
# pyproject.toml 의 name, description, [project.scripts] 수정
# src/template_python → src/my_tool 로 폴더명 변경
uv sync --extra dev
```

### TypeScript

```bash
cp -r packages/_template-typescript packages/my-tool
cd packages/my-tool
# package.json 의 name, description, bin 수정
npm install
```

### 그 외 언어

자유롭게 추가. 다만 다음만 지킬 것:
- `README.md` 에 빌드/실행 방법 한 줄.
- 빌드 산출물(`dist/`, `bin/`, `target/` 등)은 루트 `.gitignore` 에 반영되어 있는지 확인.

## 마지막 단계

새 패키지를 만들었으면 [packages/README.md](../packages/README.md) 표에 한 줄 추가.
