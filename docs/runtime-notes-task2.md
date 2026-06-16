# Runtime Notes: Axum vs. Actix Web

## The Axum / Tokio Stack
Axum sits on top of a single, global multi-threaded async runtime provided by `tokio` (via `#[tokio::main]`). 
* The application state (`AppState`) is created **once** and shared across all incoming requests. 
* Because state is shared across multiple threads, we must use thread-safe wrappers like `Arc` (Atomic Reference Counting) and synchronization primitives like `Mutex` or `RwLock` to mutate data safely.
* **The Stack:** Tokio (async execution) -> Hyper (low-level HTTP) -> Tower (middleware layers) -> Axum (routing and extraction).

## The Actix Web Stack
Actix Web utilizes a different model. It spins up multiple isolated "worker threads," and each worker runs its own single-threaded runtime.
* The application factory is instantiated **per worker**. 
* This means standard state is not shared globally; it is local to that specific worker. If you want truly global shared state in Actix, you have to explicitly define it outside the workers and pass it in as `web::Data`.

## The Danger of Blocking Work
In the Axum/Tokio model, handlers run as lightweight async tasks on a pool of OS threads. 
* If a handler executes synchronous, "blocking" work (like a heavy CPU calculation, a tight `while` loop, or a synchronous file read like `std::fs::read`), it physically holds that OS thread hostage.
* If all threads in the Tokio pool get blocked, the entire web server will freeze and stop accepting new requests, even if the CPU is mostly idle. 
* **Rule:** Heavy CPU work must be sent to a dedicated blocking pool using `tokio::task::spawn_blocking`.