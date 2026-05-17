# my_util_program

여러 util 프로그램과 MCP 서버를 한 곳에 모아 두는 **모노레포**.

- 각 프로젝트는 [packages/](packages/) 아래에 독립된 하위 패키지로 둔다.
- 언어/런타임은 패키지마다 자유 (주력은 Python, TypeScript/Node.js).
- 루트는 공용 설정([.gitignore](.gitignore), [.editorconfig](.editorconfig), [docs/](docs/))만 둔다.

## 디렉토리 구조

```
my_util_program/
├── README.md                ← 이 파일
├── .gitignore               ← 공용 ignore (Python, Node, OS)
├── .editorconfig            ← 공용 에디터 설정
├── docs/
│   └── CONVENTIONS.md       ← 패키지 추가/네이밍 규칙
└── packages/
    ├── _template-python/         ← Python 패키지 템플릿
    └── _template-typescript/     ← TypeScript 패키지 템플릿
```

## 새 패키지 추가

자세한 규칙은 [docs/CONVENTIONS.md](docs/CONVENTIONS.md) 참고. 요약:

1. 템플릿 폴더를 복사: `_template-python` → `my-tool` 형태.
2. 패키지 안의 메타데이터(`pyproject.toml` / `package.json`)에서 이름·설명을 새 패키지에 맞게 수정.
3. [packages/README.md](packages/README.md) 표에 한 줄 추가.

## 빌드/실행

루트에 통합 빌드 도구는 두지 않는다. 각 패키지 폴더에서 해당 언어의 표준 명령으로 실행:

- Python: `uv sync` → `uv run <스크립트>`
- TypeScript: `npm install` → `npm run dev`
