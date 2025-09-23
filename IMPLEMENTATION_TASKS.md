# Vectorizer - Implementation Task Board

This document tracks the assignment and status of implementation tasks for the Vectorizer project. It is derived from the `IMPLEMENTATION_CHECKLIST.md` and organized by the phases defined in `ROADMAP.md`.

**Workflow**: `Pending` -> `In Progress` -> `In Review` -> `QA Review` -> `Done`

---

## Phase 1: Foundation (Month 1)

**Objective**: Build the core engine and basic functionality. This phase is critical to establishing a solid, testable foundation.

| ID | Task | Implementer | Reviewers | Status | PR Link |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P1-INFRA-001** | **Project Setup**: Initialize Rust project, set up `Cargo.toml`, basic dependencies, and directory structure. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-INFRA-002** | **CI/CD Pipeline**: Configure basic GitHub Actions for `rustfmt`, `clippy`, and `cargo test`. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-CORE-001** | **Core Data Structures**: Implement `Vector`, `Payload`, `Collection` structs with serialization. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-CORE-002** | **VectorStore (In-Memory)**: Implement the main `VectorStore` struct with thread-safe `DashMap` for collections. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-CORE-003** | **Basic CRUD Operations**: Implement in-memory `insert`, `retrieve`, `delete` operations in `VectorStore`. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-HNSW-001** | **HNSW Index Integration**: Integrate the `hnsw_rs` crate into a new `HnswIndex` module. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-HNSW-002** | **Index Lifecycle**: Implement `add` and `search` functions within the `HnswIndex` module, connecting it to the `VectorStore`. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-PERSIST-001**| **Persistence Layer**: Implement `save` and `load` functions using `bincode` to serialize/deserialize the `VectorStore` state to a file. | TBD | TBD, TBD, TBD | `Pending` | |
| **P1-TEST-001** | **Core Unit Tests**: Achieve >95% test coverage for all data structures and `VectorStore` in-memory operations. | TBD | TBD, TBD, TBD | `Pending` | |

---

## Phase 2: Server & APIs (Month 2)

**Objective**: Create the server and external interfaces for the HiveLLM to interact with.

| ID | Task | Implementer | Reviewers | Status | PR Link |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **P2-API-001** | **Axum Setup**: Set up the `axum` web server with basic routing and state management. | TBD | TBD, TBD, TBD | `Pending` | |
| **P2-API-002** | **Collection Endpoints**: Implement REST endpoints for `CREATE`, `GET`, `DELETE` collections. | TBD | TBD, TBD, TBD | `Pending` | |
| **P2-API-003** | **Vector Endpoints**: Implement REST endpoints for `INSERT` (upsert) and `SEARCH` vectors. | TBD | TBD, TBD, TBD | `Pending` | |
| **P2-SEC-001** | **API Key Authentication**: Implement secure API key storage and middleware to protect all endpoints. | TBD | TBD, TBD, TBD | `Pending` | |
| **P2-TEST-002** | **API Integration Tests**: Create tests for all public API endpoints, including auth and error cases. | TBD | TBD, TBD, TBD | `Pending` | |

---

*Tasks for Phases 3-6 will be populated upon completion of the preceding phase.*
