# template-typescript

TypeScript 패키지 시작 템플릿.

## 사용법

1. 이 폴더를 복사: `cp -r _template-typescript my-tool`
2. `package.json` 의 `name`, `description`, `bin` 수정.
3. 의존성 설치: `npm install`.

## 개발

```bash
npm install
npm run dev      # tsx 로 즉시 실행
npm run build    # dist/ 로 컴파일
npm test         # vitest
npm run lint     # tsc --noEmit (타입 체크)
```

MCP 서버를 만든다면 `@modelcontextprotocol/sdk` 를 추가:

```bash
npm install @modelcontextprotocol/sdk
```
