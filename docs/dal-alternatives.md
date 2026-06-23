# DAL Alternatives

| Option | Pros | Cons | Protects against | Does not protect against | Fit for Ahlan |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Diesel** | Strong compile-time type safety; high performance (zero-cost abstractions); mature ecosystem; no raw strings. | Steep learning curve (custom DSL and macros); slower compile times; can be complex to express highly dynamic SQL. | SQL injection; schema and type mismatches (at compile time). | N+1 query problems (unless manually optimized); the cognitive overhead of learning a DSL instead of raw SQL. | No. Hides exact SQL behavior behind ORM abstractions. |
| **SeaORM** | Native async support; easy dynamic query building; multi-database support; familiar ActiveRecord/Eloquent-like API. | Heavier runtime overhead compared to SQLx or Diesel; relies heavily on macros. | SQL injection. | Strict compile-time schema mismatches (relies more on generated entities); abstraction leaks on complex queries. | No. Hides database operations behind heavy abstractions. |
| **SQLx checked queries** (`query!`) | Write pure, raw SQL; compile-time verification against a live DB schema; lightweight and fast. | Raw SQL strings are embedded directly in Rust code; refactoring schemas means hunting down these strings in app code. | SQL injection; invalid SQL syntax; type mismatches. | **"SQL drift"** from the application structure; missed query updates when refactoring; boilerplate code. | No. SQL strings still pollute Rust application logic. |
| **SQL-first code generation pattern** | SQL lives in isolated, named `.sql` files; host language gets perfectly typed bindings; separation of concerns. | Requires a code generation step in the build process; generated code still needs to be wrapped in a DAL. | SQL string drift in app code; type mismatches between SQL results and application structs. | Missing regeneration step when SQL changes; architecture failures (if generated code is used in business logic). | Matches the core philosophy. |
| **Cornucopia** | Excellent performance; perfectly typed Rust structs matching Postgres queries; supports complex PG features natively. | Postgres only; requires a live schema for generation; less mature than Diesel/SQLx. | SQL injection; schema/type mismatches; SQL strings polluting Rust logic. | Outdated generated code (if generation step is missed); lack of domain boundaries (you still need to build a DAL). | **Ahlan default.** |

## What Ahlan-commerce will work with and why

Ahlan-commerce uses **SQL-First Generated Code with Cornucopia**.

### Why this is the chosen pattern:

1. **SQL Remains Reviewable as SQL:**
   Ahlan-commerce values visibility over abstraction. Rather than using an ORM like Diesel or SeaORM that hides what the database is actually executing behind a custom Rust API, the SQL-first pattern allows developers to write and review pure, isolated `.sql` files. Reviewers can easily inspect the SQL operations and the Rust boundaries separately.

2. **Fixes "Raw SQL Drift":**
   Using plain `SQLx` checked queries means burying SQL strings directly inside application code (like your Axum handlers). This inevitably leads to "raw SQL drift," where code can compile but behave incorrectly because embedded strings were missed during refactoring, or domain code ends up learning too many table details. Cornucopia reads isolated `.sql` files and generates strongly typed Rust functions. If the schema or query changes, the generated Rust code updates, and the Rust compiler immediately catches any mismatches.

3. **Enforces a Clear DAL Boundary:**
   Cornucopia generates *persistence primitives*, not application policy. Ahlan-commerce takes these typed generated queries and places them behind a dedicated Data Access Layer (DAL) package. This boundary ensures that your HTTP handlers and domain orchestration never touch raw SQL or the generated bindings directly, keeping concerns perfectly separated.

4. **Separation of Migration and Execution:**
   Ahlan-commerce uses two distinct tools for the database that should not be confused:
   - **Atlas** manages the database schema migration history.
   - **Cornucopia** provides typed Rust access to named SQL queries.

   Neither tool replaces the other, and together they provide a robust, type-safe, and highly visible persistence strategy without the opacity of a traditional ORM.
