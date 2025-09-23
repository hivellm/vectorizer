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

## Phase 3: Testing & Quality (Month 3)

**Objective**: Ensure system reliability, performance, and robustness before adding more features.

| ID               | Task                                                                    | Implementer | Reviewers     | Status    | PR Link |
| :--------------- | :---------------------------------------------------------------------- | :---------- | :------------ | :-------- | :------ |
| **P3-TEST-001**  | **Integration Tests**: Write end-to-end tests for all core user flows.      | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P3-TEST-002**  | **Property-Based Testing**: Add `proptest` for core data structures.       | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P3-PERF-001**  | **Benchmark Suite**: Implement `criterion` benchmarks for core operations. | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P3-PERF-002**  | **Load Testing Framework**: Set up a basic load testing environment.      | TBD         | TBD, TBD, TBD | `Pending` |         |

---

## Phase 4: Client SDKs (Month 4)

**Objective**: Provide high-quality, easy-to-use SDKs for Python and TypeScript.

| ID               | Task                                                                    | Implementer | Reviewers     | Status    | PR Link |
| :--------------- | :---------------------------------------------------------------------- | :---------- | :------------ | :-------- | :------ |
| **P4-SDK-PY-001**| **Python SDK (HTTP Client)**: Create a Python client using `requests`/`httpx`. | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P4-SDK-PY-002**| **PyO3 Bindings (Optional)**:spike native bindings for performance-critical parts. | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P4-SDK-TS-001**| **TypeScript SDK (HTTP Client)**: Create a TypeScript client using `axios`/`fetch`.| TBD         | TBD, TBD, TBD | `Pending` |         |
| **P4-SDK-TS-002**| **SDK Packaging**: Package and prepare for distribution (PyPI, npm).        | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P4-TEST-003**  | **SDK Integration Tests**: Write integration tests for both SDKs.          | TBD         | TBD, TBD, TBD | `Pending` |         |

---

## Phase 5: Production Features (Month 5)

**Objective**: Implement operational tools required for production environments.

| ID               | Task                                                              | Implementer | Reviewers     | Status    | PR Link |
| :--------------- | :---------------------------------------------------------------- | :---------- | :------------ | :-------- | :------ |
| **P5-FEAT-001**  | **Dashboard**: Implement the localhost-only web dashboard.          | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P5-FEAT-002**  | **CLI Tool**: Implement the `clap`-based CLI tool for administration. | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P5-FEAT-003**  | **Configuration System**: Implement full `config.yml` parsing.   | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P5-OPS-001**   | **Monitoring & Metrics**: Implement Prometheus metrics exporter.      | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P5-OPS-002**   | **Structured Logging**: Implement `tracing` for structured logs.    | TBD         | TBD, TBD, TBD | `Pending` |         |

---

## Phase 6: Experimental Features (Month 6+)

**Objective**: Explore advanced optimizations and features after the core system is stable.

| ID               | Task                                                              | Implementer | Reviewers     | Status    | PR Link |
| :--------------- | :---------------------------------------------------------------- | :---------- | :------------ | :-------- | :------ |
| **P6-EXP-001**   | **Quantization**: Implement PQ, SQ, and Binary quantization.        | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P6-EXP-002**   | **UMICP Integration**: Spike integration with UMICP.              | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P6-EXP-003**   | **CUDA Acceleration**: Spike GPU-accelerated search with CUDA.        | TBD         | TBD, TBD, TBD | `Pending` |         |
| **P6-EXP-004**   | **LangChain Integration**: Implement LangChain VectorStore classes.   | TBD         | TBD, TBD, TBD | `Pending` |         |
