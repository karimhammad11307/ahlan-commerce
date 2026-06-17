# Developer Daily Log: June 15, 2026

## Morning Session: Rust + MongoDB (Without Frameworks)
* **Goal**: Understand raw MongoDB CRUD operations and connections in Rust.
* **Challenges**: Managing the asynchronous connection lifecycle (`mongodb::Client`) and handling manual BSON serialization/deserialization with `serde` without framework abstractions.
* **Outcome**: Gained insight into driver thread safety and raw query execution, which was initially challenging but highly educational.

---Arc<Mutex

## Afternoon Session: Ahlan Commerce Tasks
* **Axum Server Setup**: Configured an HTTP server using Axum and Tokio in `tasks/`.
* **In-Memory State**: Shared product data across handlers using a thread-safe wrapper: `<Vec<catalog::Product>>>`.
* **API Endpoints**: 
  * `GET /health` — Simple health check.
  * `GET /api/products` — Fetches the product list.
  * `POST /api/products` — Adds new products to the in-memory database.
* **Type Inference Fix**: Resolved the Router `E0282` compiler error by explicitly annotating the type of the `Router`.
* **Architectural Research**: Wrote comparisons between Axum/Tokio and Actix Web runtime models (concerning thread safety and safe management of blocking sync tasks in async runtime).

---

## Files Updated
* [tasks/src/main.rs](file:///home/karimhammad/Workspace/Vscode%20Projects/ahlan-commerce/tasks/src/main.rs) — Route handlers, server setup, and state management.
* [tasks/src/catalog.rs](file:///home/karimhammad/Workspace/Vscode%20Projects/ahlan-commerce/tasks/src/catalog.rs) — Product struct modeling and testing shell.
* [docs/runtime-notes-task2.md](file:///home/karimhammad/Workspace/Vscode%20Projects/ahlan-commerce/docs/runtime-notes-task2.md) — Runtime research notes.
* [docs/learning-log.md](file:///home/karimhammad/Workspace/Vscode%20Projects/ahlan-commerce/docs/learning-log.md) — Initialized learning log.
