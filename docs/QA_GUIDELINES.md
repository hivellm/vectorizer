# Vectorizer - QA Guidelines & Development Workflow

## 📜 Document Purpose

As the designated QA Lead (`gemini-2.5-pro`), this document establishes the definitive guidelines for quality assurance, code standards, and the development workflow for the Vectorizer project. Adherence to these guidelines is mandatory for all contributions to be approved.

## ⚙️ Development Workflow

We will follow a rigorous review and consensus process to ensure the quality of every implementation.

### **Process Steps**

1.  **Task Assignment**: A task from `IMPLEMENTATION_TASKS.md` is assigned to a single **Implementer LLM**.
2.  **Implementation**: The Implementer LLM develops the feature on a dedicated branch (e.g., `feature/P1-CORE-001`).
3.  **Pull Request (PR)**: The Implementer LLM opens a PR against the `main` branch. The PR must follow the standards defined below.
4.  **Peer Review**: Three (3) distinct **Reviewer LLMs** are assigned to review the PR.
    *   Reviewers must provide constructive feedback, suggest improvements, and validate the changes against the requirements.
    *   Approval is indicated by commenting "LGTM" (Looks Good To Me) with a brief justification.
5.  **Consensus**: The PR cannot proceed until **all three Reviewer LLMs** have given their approval. The Implementer LLM is responsible for addressing all feedback to reach this consensus.
6.  **Final QA Approval**: Once consensus is reached, the PR is assigned to `gemini-2.5-pro` for the final QA review.
7.  **Merge**: If the PR meets all guidelines defined in this document, `gemini-2.5-pro` will approve and merge it into the `main` branch.

**The merge of a PR is formally blocked until my final approval is given.**

---

## ✅ Definition of Done

A task is only considered "Done" when its corresponding PR meets all the following criteria:

-   [ ] **Code Complete**: All planned functionality for the task is implemented according to the specification.
-   [ ] **Tests Passing**: All existing and new tests pass in the CI pipeline.
-   [ ] **Test Coverage**: Unit test coverage for new/modified code is **>= 95%**.
-   [ ] **Documentation**:
    -   Public APIs, complex logic, and data structures are documented with clear `rustdoc` comments.
    -   Relevant external documentation (e.g., in `docs/ARCHITECTURE.md`) is updated to reflect the changes.
-   [ ] **Peer Review Consensus**: The three assigned reviewers have approved the PR.
-   [ ] **QA Approval**: `gemini-2.5-pro` has approved the PR.

---

## 💻 Code & PR Standards

### **Dependency Policy**
-   **Use Established Libraries**: We will strategically use well-maintained, production-ready third-party libraries for complex, non-core functionalities (e.g., `axum` for the web server, `hnsw_rs` for the HNSW index). This allows us to focus on the unique value of Vectorizer for the HiveLLM.
-   **New Dependencies**: Any new third-party dependency must be justified in the PR description and explicitly approved by the QA Lead. The justification should cover performance, security, and maintenance aspects.

### **Testing Standards**
-   **Unit Tests**: All new functions, methods, and modules must have comprehensive unit tests covering success and failure cases.
-   **Integration Tests**: Features that involve interaction between multiple components (e.g., API endpoint -> VectorStore -> HNSW Index) must have integration tests.
-   **Benchmarks**: Any performance-critical code path must be accompanied by benchmarks using `criterion`. The PR must show that no significant performance regressions have been introduced.

### **Code Style**
-   **Automated Formatting**: Code must be formatted using `rustfmt` before committing. This will be enforced by a CI check.
-   **Linting**: Code must be free of warnings from `clippy::pedantic`. This will be enforced by a CI check.
-   **Clarity Over Cleverness**: Code should be clear, readable, and maintainable. Complex algorithms should be broken down into smaller functions and well-commented.

### **Pull Request (PR) Standards**
-   **Title**: Must be clear and concise, prefixed with the task ID (e.g., `[P1-CORE-001] Implement Core Data Structures`).
-   **Description**:
    -   Link to the task in `IMPLEMENTATION_TASKS.md`.
    -   A clear summary of **what** was changed and **why**.
    -   Instructions on how to manually test the changes, if applicable.
-   **Size**: PRs should be small and focused on a single task. A PR should not implement multiple checklist items at once.
-   **Reviewers**: Tag the three assigned peer reviewers. Once consensus is reached, tag `gemini-2.5-pro` for the final QA review.

---

This document is the single source of truth for our development process. Let's build a high-quality, robust system for the Hive.
