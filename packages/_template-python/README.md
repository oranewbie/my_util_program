# template-python

Python 패키지 시작 템플릿.

## 사용법

1. 이 폴더를 복사: `cp -r _template-python my-tool`
2. `pyproject.toml` 의 `name`, `description`, `[project.scripts]` 수정.
3. `src/template_python/` 폴더명을 `src/my_tool/` 로 변경 (Python은 snake_case).
4. 의존성 추가: `uv add <패키지>` 혹은 `pyproject.toml` 직접 편집.

## 개발

```bash
uv sync --extra dev     # 가상환경 생성 + 의존성 설치
uv run template-python  # 실행
uv run pytest           # 테스트
uv run ruff check .     # 린트
```
