---
description: TypeScript/Next.js conventions for the web dashboard
globs: ["services/web/**/*.ts", "services/web/**/*.tsx", "services/web/**/*.js"]
---

# TypeScript / Next.js Conventions

## Next.js 16 + React 19
- Use App Router (`app/` directory)
- Server Components by default, `'use client'` only when needed
- Server Actions for mutations
- Proper loading.tsx and error.tsx boundaries

## TypeScript
- Strict mode enabled
- No `any` types - use proper typing or `unknown` with type guards
- Interface for object shapes, Type for unions/intersections
- Zod for runtime validation at API boundaries

## Styling
- Tailwind CSS 4
- No inline styles - use Tailwind classes
- Component-scoped styles when needed

## State Management
- React 19 hooks (useState, useReducer)
- Server state via fetch + cache
- No unnecessary client-side state

## Error Handling
- try-catch with typed errors
- Error boundaries for component trees
- User-friendly error messages

## Performance
- Use `React.memo`, `useMemo`, `useCallback` only when measured
- Image optimization via next/image
- Dynamic imports for heavy components
- Minimize client-side JavaScript

## No Debug Output
- No `console.log` in production code
- Use proper logging if needed
