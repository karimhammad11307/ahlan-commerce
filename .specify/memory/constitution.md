<!-- 
Sync Impact Report:
Version change: 1.0.0 → 1.1.0
Modified principles: 
- Initial definition based on user prompt.
Added sections: Core Principles, Governance
Removed sections: Template placeholders.
Templates requiring updates: ✅ None pending.
-->
# Ahlan-Commerce Constitution

## Core Principles

### I. Technology Stack
The project MUST use Rust, Axum for the web framework, and PostgreSQL for the database. Database access MUST be managed exclusively via Cornucopia.

### II. Thin Handlers
Axum route handlers MUST be thin. They are responsible ONLY for extracting request data, delegating to the business logic layer, and formatting the response. 

### III. Separation of Concerns
NO business logic is permitted inside route handlers. All business rules, validations, and domain logic MUST be encapsulated in dedicated service modules or domain objects.

### IV. Standardized Errors
All application errors MUST return standard JSON payloads. Handlers MUST NOT return raw text errors, HTML errors, or generic server panics to the client. The error format must remain consistent across the entire API surface.

## Governance

This Constitution supersedes all other practices. All Pull Requests and code reviews MUST verify compliance with these core architectural constraints. 

**Version**: 1.1.0 | **Ratified**: 2026-06-25 | **Last Amended**: 2026-06-26
