# Daily Log - June 23, 2026

## What I Implemented Today
Today was a major step towards building the frontend ecosystem and decoupling heavy backend processes. I completed Chapters 10 and 11, introducing a modern React UI and a durable background worker pattern.

### 1. Chapter 10: TanStack React Admin UI
- **SPA Scaffold:** Initialized a strict React + TypeScript single-page application using Vite in `apps/admin`.
- **Strict Separation of Concerns:** 
  - Configured **TanStack Router** to manage URL and screen state strictly via file-based routing.
  - Configured **TanStack Query** to manage server state, caching, data fetching, and automatic refetching.
- **Thin GraphQL Wrapper:** Built a thin client layer using pure `fetch` to talk to our API's `/graphql` endpoint, avoiding heavy libraries like Apollo Client.
- **Integration:** Updated our `mprocs.yaml` ecosystem so running `make start` simultaneously boots the DB, API, and the new Admin React UI.

### 2. Chapter 11: The Background Worker
- **Paradigm Shift:** Moved away from synchronous HTTP processing for heavy tasks (like importing products) to a decoupled model: API records a "Job", Worker processes it later.
- **Database Schema:** Added the `import_jobs` table tracking status (`queued`, `running`, `succeeded`, `failed`), attempts, and errors.
- **Cornucopia Queries:** Used `FOR UPDATE SKIP LOCKED` to allow multiple workers to safely acquire jobs concurrently without double-processing.
- **Worker Process:** Created a standalone Rust binary (`apps/worker`) that polls the database every 2 seconds, parses JSON files, attempts product creation via domain logic, and safely handles retries on failures.

---

## What I Learned
1. **Chatting with AI for Architecture & Mentorship:** I leveraged AI not just for code, but to break down chapters and provide mentorship-style explanations. We discussed *why* we chose the TanStack ecosystem and deeply explored the philosophy of decoupling synchronous requests into durable background jobs to survive crashes and timeouts.
2. **TanStack Magic:** I learned how TanStack Router purely dictates UI based on URLs, while TanStack Query intelligently caches data. The `refetchOnWindowFocus` feature is incredible—it automatically refetches data in the background if the user clicks away and comes back, eliminating the need for complex websockets in many cases!
3. **Database Concurrency:** Learned how `FOR UPDATE SKIP LOCKED` works in PostgreSQL to easily build a reliable job queue directly inside the database.

---

## Issues Faced & Fixed
- **VSCode TypeScript Chaos:** I had IDE errors failing to find `@tanstack/react-router` and `import.meta.env` because VSCode couldn't resolve `apps/admin/node_modules`. 
  - *Fix:* Created a root `tsconfig.json` with project references and explicitly added `vite/client` types to the admin app.
- **5,000+ Git Commits Nightmare:** Git accidentally staged the entire `node_modules` and `dist` folders for the admin app.
  - *Fix:* Thoroughly updated the root `.gitignore` to explicitly ignore `/apps/admin/node_modules`, `dist`, and generated route files, reducing the staged changes to just 16 source files.
- **Cornucopia Syntax Strictness:** Ran into multiple `make cornucopia-generate` failures. 
  - *Fix:* Learned that Cornucopia requires empty parentheses `()` for zero-parameter queries, requires the `?` symbol to map `LIMIT 1` to `Option<T>`, and can struggle with nullability inference (which I fixed by mapping an empty `String` to `None` in Rust).
- **Rust Ownership Battles:** Hit a "borrow of moved value" compiler error when handling a failed job because I passed the `job` object to a function and then tried to use its properties in a log message.
  - *Fix:* Appended `.clone()` to safely pass a copy of the job.

---

## Important Links & Docs
### Videos to Revise:
*   **TanStack Query - How to become a React Query**
    *   https://www.youtube.com/watch?v=mPaCnwpFvZY
*   **TanStack Router Tutorial:**
    *   https://youtu.be/KcKkwF55Pes?si=ff5V5biqxc9b_pXW

### Core Documentation:
*   **TanStack Query Docs:** [https://tanstack.com/query/latest](https://tanstack.com/query/latest)
*   **TanStack Router Docs:** [https://tanstack.com/router/latest](https://tanstack.com/router/latest)
