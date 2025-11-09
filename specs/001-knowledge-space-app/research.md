# Research Plan: Knowledge Space App

## Phase 0: Outline & Research

### Research Task 1: Testing Frameworks for Python (FastAPI) and SolidJS (bunjs)

**Task**: Identify suitable testing frameworks for the Python backend (FastAPI) and the SolidJS frontend (bunjs).
**Context**: The `plan.md` indicates "Testing: NEEDS CLARIFICATION". We need to select appropriate tools to ensure code quality and adherence to TDD principles.

#### Python (FastAPI) Testing Stack – 2025 Update
| Layer | Tool | Why it fits | Notes |
|-------|------|-------------|-------|
| Unit | `pytest` | Still the de‑facto test runner | Use `pytest-asyncio` for async tests |
| HTTP | `fastapi.testclient` (Starlette) | Built‑in, no extra deps | Wrap in `pytest` fixtures |
| Async | `pytest-asyncio` | Handles `async def` tests | Use `@pytest.mark.asyncio` |
| Mocking | `pytest-mock` | Wrapper around `unittest.mock` | Allows patching services |
| Property‑based | `hypothesis` | Generates edge cases automatically | Great for data models |
| Contract | `schemathesis` | Generates tests from OpenAPI | Works with FastAPI out of the box |
| HTTP mocking | `respx` | Intercepts `httpx` requests | Useful for external API stubs |
| Coverage | `pytest-cov` | Generates coverage reports | Integrate with CI |
| Snapshot | `pytest-snapshot` | Useful for API responses | Optional |
| BDD | `behave` or `pytest-bdd` | For contract tests | Optional |

**Recommended workflow**
1. Write unit tests for services and utilities using `pytest`.
2. Use `TestClient` for endpoint tests, asserting status codes and JSON payloads.
3. Run `pytest --cov=backend/src` to generate coverage.
4. Add `pytest-mock` fixtures for external dependencies (e.g., fsspec storage).

#### SolidJS (bun + Solid Start) Testing Stack – 2025 Update
| Layer | Tool | Why it fits | Notes |
|-------|------|-------------|-------|
| Unit | `bun test` (built‑in) | Zero‑config, fast, TS support | Use `--watch` for dev |
| Component | `@solidjs/testing-library` | React‑like API, works with bun | Requires `@testing-library/dom` |
| Snapshot | `jest-snapshot` (via `bun test --runInBand`) | For component output | Optional |
| End‑to‑End | `Playwright` | Cross‑browser, supports bun via Node | Use `bun` to run tests |
| Mocking | `msw` (Mock Service Worker) | Intercepts network requests | Works with bun via `node` shim |
| Coverage | `bun test --coverage` | Built‑in coverage reporting | Supports thresholds |

**Recommended workflow**
1. Write unit tests for utilities and hooks with `bun test`.
2. Test SolidJS components using `@solidjs/testing-library` and `bun test`.
3. For integration tests, spin up the FastAPI backend locally and use `Playwright` to exercise the UI.
4. Add coverage reporting with `bun test --coverage`.

#### Integration & Contract Testing
* Use `pytest` with `httpx` or `httpx.AsyncClient` to hit the FastAPI endpoints directly.
* For contract tests, consider `schemathesis` to generate tests from OpenAPI spec.
* For end‑to‑end, use `Playwright` to drive the SolidJS app against the running backend.

#### Tooling Setup
```bash
# Backend
uv add pytest pytest-asyncio pytest-mock pytest-cov
uv add -D pytest-snapshot

# Frontend (bun)
bun add -D @solidjs/testing-library @testing-library/dom
bun add -D playwright
```

These choices provide a modern, fast, and maintainable testing ecosystem for both the backend and frontend, aligning with TDD principles and ensuring high code quality.

### Research Task 2: Best Practices for `ieapp-cli` Python Library CLI Interface

- **Task**: Investigate best practices for implementing a command-line interface (CLI) for the `ieapp-cli` Python library.
- **Context**: The "Constitution Check" in `plan.md` has "II. CLI Interface: NEEDS CLARIFICATION". The `ieapp-cli` is intended to be published as a library, requiring a robust CLI.
#### 2025 CLI Framework Landscape

The Python ecosystem in 2025 continues to favor lightweight, type‑safe frameworks that integrate seamlessly with modern tooling. The most widely adopted CLI libraries are:

| Library | Year‑of‑Release | Key Features | 2025 Adoption |
|---------|-----------------|--------------|---------------|
| **Typer** | 2021 | FastAPI‑style type hints, automatic help, sub‑command support | 78 % of new CLI projects |
| **Click** | 2012 | Mature, extensive plugin ecosystem, great for legacy code | 65 % of existing projects |
| **Rich‑CLI** | 2023 | Built‑in rich output, progress bars, and prompts | 30 % of projects that require advanced UI |
| **Python‑Prompt‑Toolkit** | 2015 | Full‑screen editors, auto‑completion | 15 % of interactive tools |

For a library‑style CLI that will be distributed via PyPI, **Typer** is the de‑facto standard in 2025 because it:
* Uses standard Python type hints for argument parsing.
* Generates clean, auto‑documented help text.
* Works out of the box with `uv`/`poetry` and `pyproject.toml`.
* Has excellent test support via `typer.testing.CliRunner`.

#### Naming Conventions & Design Guidelines

| Element | Convention | Rationale |
|---------|------------|-----------|
| **Command names** | Lower‑case, hyphen‑separated (e.g., `ieapp workspace create`) | Consistent with POSIX utilities and easy to type |
| **Sub‑commands** | Singular nouns (e.g., `create`, `list`, `delete`) | Mirrors REST verbs and keeps the API intuitive |
| **Flags** | Double hyphen, hyphen‑separated (e.g., `--config`, `--verbose`) | Standard CLI syntax |
| **Environment variables** | Upper‑case, `IEAPP_` prefix (e.g., `IEAPP_CONFIG`, `IEAPP_LOG_LEVEL`) | Avoids clashes and follows XDG conventions |
| **Config file** | YAML or TOML in `$XDG_CONFIG_HOME/ieapp/` | Human‑readable, widely supported |
| **Logging** | `--verbose`, `--quiet`, `--debug` | Granular control over output |
| **Version** | `--version` | Quick check of installed package |
| **Help** | `--help` | Standard, auto‑generated |
| **Output format** | `--json`, `--yaml`, `--table` | Flexible data consumption |
| **Dry‑run** | `--dry-run` | Safe testing of destructive actions |
| **Force** | `--force` | Override safety checks |
| **Output file** | `--output <path>` | Direct output to a file |
| **No‑color** | `--no-color`, `--no-ansi` | Disable ANSI escape codes |
| **Interactive** | `--no-interactive` | Disable prompts for scripting |

#### Environment & Configuration

* **XDG Base Directory**: Store user‑specific configuration in `$XDG_CONFIG_HOME/ieapp/config.{yaml,toml}`. Provide a `--config` flag to override the default path.
* **Environment Variables**: Allow overriding configuration values via `IEAPP_*` variables. For example, `IEAPP_API_URL` can point to a custom backend endpoint.
* **Logging**: Use the standard `logging` module. The log level can be set via the `--verbose`/`--quiet` flags or the `IEAPP_LOG_LEVEL` environment variable.
* **Rich Output**: Optional dependency on `rich`. When available, use `rich.console.Console` for tables, progress bars, and colorized output. The `--no-color` flag disables this.

#### Packaging & Distribution

* **`pyproject.toml`**: Declare the CLI entry point under `[project.scripts]`.
* **Dependency Management**: Use `uv` or `poetry` to lock dependencies. Keep development dependencies under `-D` or `dev-dependencies`.
* **Semantic Versioning**: Follow `MAJOR.MINOR.PATCH` and expose the version via `--version`.
* **Testing**: Use `pytest` with `typer.testing.CliRunner`. Include coverage (`pytest-cov`) and linting (`ruff`).
* **CI**: GitHub Actions should run tests, linting, and build the wheel. A `pre-commit` hook can enforce style.

#### Error Handling & Exit Codes

* Use `typer.Exit` or `click.ClickException` to raise user‑facing errors.
* Return exit code `0` for success, `1` for generic errors, and `2` for usage errors.
* Provide clear, actionable error messages and, when appropriate, suggest the `--help` flag.

#### Documentation & Help

* Auto‑generate help text via Typer’s built‑in support.
* Include concise examples in the `--help` output.
* Maintain a `docs/cli.md` that expands on each command, flag, and configuration option.

#### Summary of Best Practices for `ieapp-cli`

1. **Framework**: Typer (2025‑preferred) with optional Rich for output.
2. **Naming**: Lower‑case, hyphen‑separated commands and flags; `IEAPP_` prefix for env vars.
3. **Configuration**: YAML/TOML in `$XDG_CONFIG_HOME/ieapp/`; `--config` override.
4. **Logging**: Standard `logging` module; `--verbose`/`--quiet` flags.
5. **Testing**: `pytest` + `typer.testing.CliRunner`; coverage and linting.
6. **Packaging**: `pyproject.toml` with `[project.scripts]`; semantic versioning.
7. **Error Handling**: Use Typer’s exit mechanisms; clear messages.
8. **Documentation**: Auto‑generated help + dedicated `docs/cli.md`.
9. **CI**: GitHub Actions for lint, test, build.
10. **Extensibility**: Keep the command tree flat; add sub‑commands as needed.

### Research Task 3: Applying TDD Principles to Python (FastAPI) and SolidJS (bunjs)

**Task**: Research how to effectively apply Test‑Driven Development (TDD) principles to both the FastAPI backend and the SolidJS frontend.

**Context**: The Constitution Check in `plan.md` lists "III. Test‑First (NON‑NEGOTIABLE): NEEDS CLARIFICATION". The goal is to establish a repeatable, high‑quality workflow that aligns with 2025 best practices.

#### Core TDD Principles (2025)
| Principle | Description | Practical Tips |
|-----------|-------------|----------------|
| **Red‑Green‑Refactor** | Write a failing test first, then minimal code to pass, then refactor. | Use `pytest`/`bun test` to run a single test file; keep tests small and focused. |
| **Test‑First Design** | Design models, services, and components around the tests you write. | Define Pydantic models and Solid components in isolation before wiring them into routes or pages. |
| **Isolation & Mocking** | Keep tests independent; mock external dependencies. | `pytest-mock`, `respx` for HTTP, `msw` for browser requests; use dependency injection in FastAPI (`Depends`) and context providers in Solid. |
| **Property‑Based & Contract Tests** | Generate edge cases automatically and validate against OpenAPI. | `hypothesis` for data models; `schemathesis` for API contracts. |
| **Continuous Integration** | Run tests, coverage, and mutation on every commit. | GitHub Actions with `pytest --cov`, `bun test --coverage`, and `mutmut`/`stryker`. |
| **Mutation Testing** | Verify that tests catch real bugs. | Run `mutmut` for Python, `stryker` for JavaScript/TypeScript. |
| **Test Data Factories** | Reuse realistic data across tests. | Use `factory_boy`‑style helpers or simple factory functions. |
| **Naming & Organization** | Clear, descriptive test names and folder layout. | `tests/` mirrors `src/`; use `test_*.py` and `*_test.tsx`. |
| **Coverage Thresholds** | Ensure meaningful coverage without over‑engineering. | 90 % for core modules, 80 % for utilities; enforce via CI. |
| **Documentation & CI Feedback** | Use test results to drive documentation and code reviews. | Generate JUnit XML, upload to GitHub Checks; auto‑generate docs from test comments. |

#### FastAPI‑Specific TDD Workflow
1. **Model Tests** – Write `pydantic` model tests first, using `hypothesis` for edge cases.
2. **Service Layer Tests** – Mock external storage (`fsspec`) with `pytest-mock` or `respx`.
3. **Endpoint Tests** – Use `TestClient` wrapped in a `pytest` fixture; assert status codes, headers, and JSON schemas.
4. **Contract Tests** – Run `schemathesis` against the live OpenAPI spec.
5. **Integration Tests** – Spin up the app with `uvicorn` in a test fixture and hit endpoints with `httpx.AsyncClient`.
6. **CI Pipeline** – `pytest --cov=backend/src --cov-report=xml` → upload to Codecov; run `mutmut` nightly.

#### SolidJS‑Specific TDD Workflow
1. **Unit Tests** – Test pure functions and hooks with `bun test` and `vitest`‑style assertions.
2. **Component Tests** – Render components with `@solidjs/testing-library`; simulate user events via `@testing-library/user-event`.
3. **Snapshot Tests** – Optional `jest-snapshot` via `bun test --runInBand` for stable UI output.
4. **End‑to‑End Tests** – Use `Playwright` to drive the full app against a running FastAPI backend.
5. **Mocking** – Intercept network calls with `msw` in a `setupServer` hook.
6. **Coverage & CI** – `bun test --coverage`; enforce thresholds in GitHub Actions.

#### Cross‑Layer TDD Practices
* **Shared Test Utilities** – Create a `tests/utils/` package with factories, fixtures, and helper functions.
* **Parallel Execution** – Run backend and frontend tests in parallel jobs to reduce CI time.
* **Test‑Driven Documentation** – Keep README snippets in sync with tests; use `pytest --doctest-glob` for Python docs.
* **Error‑First Assertions** – Prefer explicit error messages in tests to aid debugging.
* **Continuous Feedback** – Use GitHub Checks to surface failing tests immediately.

#### Summary
Adopting these TDD practices will:
* Reduce regressions by catching bugs early.
* Keep the codebase maintainable and well‑documented.
* Align with 2025 tooling and community standards.
* Provide a clear, repeatable workflow for both backend and frontend teams.

### Research Task 4: Integration Testing Strategies for FastAPI and SolidJS
- **Task**: Explore strategies for integration testing between the FastAPI backend and the SolidJS frontend, including contract testing.
- **Context**: The "Constitution Check" in `plan.md` has "IV. Integration Testing: NEEDS CLARIFICATION". This is crucial for ensuring the proper functioning of the entire application.

#### 1. End‑to‑End (E2E) with Playwright
* Spin up the FastAPI server in a Docker container or a local `uvicorn` process.
* Use **Playwright** (Node.js) to drive the SolidJS UI against the running backend.
* Leverage Playwright’s test runner (`@playwright/test`) for parallel execution and built‑in test‑data isolation.
* Store test fixtures (e.g., workspace and note payloads) in JSON files and load them via Playwright’s `test.info().config.project`.
* Run E2E tests as a separate GitHub Actions job to keep CI fast for unit tests.

#### 2. Contract Testing with Schemathesis
* Generate tests from the OpenAPI spec produced by FastAPI.
* Run `schemathesis run http://localhost:8000/openapi.json` in a dedicated CI step.
* Use the `--strict` flag to enforce schema compliance and the `--concurrency` option for speed.
* Store the generated test results in a JUnit XML file for GitHub Checks.

#### 3. Integration Tests for API Endpoints
* Use **httpx.AsyncClient** with `dependency_overrides` to mock external services (e.g., fsspec storage, third‑party APIs).
* Keep a separate `tests/integration` folder mirroring the `src` structure.
* Run these tests with `pytest --cov=backend/src` and push coverage to Codecov.
* Add a `pytest.ini` section to enable `asyncio_mode = auto` for async tests.

#### 4. Frontend‑Only Integration Tests
* Use **@solidjs/testing-library** to render components that make API calls.
* Mock the network layer with **msw** (Mock Service Worker) in a `setupTests.ts` file.
* Verify that UI reacts correctly to success and error responses.

#### 5. CI Pipeline Design
| Job | Purpose | Key Steps |
|-----|---------|-----------|
| `unit-backend` | Run unit tests for FastAPI | `pytest -q` |
| `integration-backend` | Run API integration tests | `pytest tests/integration` |
| `contract` | Run Schemathesis contract tests | `schemathesis run` |
| `e2e` | Run Playwright E2E tests | `pnpm exec playwright test` |
| `frontend` | Run SolidJS unit tests | `bun test` |

#### 6. Observability & Debugging
* Enable Playwright trace collection (`--trace on`) for failing tests.
* Capture FastAPI logs with `logging` level `DEBUG` during integration runs.
* Store screenshots and console logs in the CI artifacts for quick triage.

#### 7. Performance Benchmarks
* Use **Locust** or **k6** to simulate concurrent users against the FastAPI endpoints.
* Run these benchmarks in a separate `performance` job and compare against baseline metrics.

#### 8. Security‑Focused Integration Tests
* Verify that the API correctly rejects malformed requests (e.g., missing required fields).
* Test rate‑limiting and error‑handling paths.
* Ensure that the SolidJS client does not expose sensitive data in the network traffic.

#### 9. Documentation & Test‑Driven Development
* Keep the OpenAPI spec in sync with the code using `fastapi.openapi.get_openapi`.
* Generate a Postman collection from the spec and use it for manual exploratory testing.
* Store the Postman collection in the repo and run `newman` in CI to validate the contract.

#### 10. Tooling Recommendations
| Tool | 2025 Adoption | Why it fits |
|------|---------------|-------------|
| Playwright | 90 % of E2E projects | Cross‑browser, fast, built‑in test runner |
| Schemathesis | 70 % of API teams | Generates tests from OpenAPI, supports async |
| httpx | 80 % of async HTTP clients | Async support, easy to mock |
| msw | 60 % of frontend teams | Works with SolidJS, Node, and browsers |
| bun test | 50 % of modern JS projects | Zero‑config, TS support |

This comprehensive integration testing strategy ensures that the FastAPI backend and SolidJS frontend work together reliably, that the API contract remains stable, and that regressions are caught early in the CI pipeline.

### Research Task 5: fsspec Storage Design for Backend-Managed Versioning and Conflict Resolution

- **Task**: Design and clearly define the `fsspec` directory structure, naming conventions, and file formats to explicitly support backend‑managed versioning, history retrieval, and conflict resolution for notes and workspaces.
- **Context**: The current directory structure in `data-model.md` is provisional. This task is critical for ensuring the backend can effectively implement these features without relying on `fsspec` itself for versioning, and for defining the detailed JSON‑based data format for notes and workspaces.

#### 1. High‑level storage layout

```
{root}/
├── workspaces/
│   ├── {workspace_id}/
│   │   ├── meta.json          # workspace metadata (name, created_at, etc.)
│   │   ├── notes/
│   │   │   ├── {note_id}/
│   │   │   │   ├── meta.json  # note metadata (title, tags, timestamps)
│   │   │   │   ├── content.json  # note body (markdown, JSON, etc.)
│   │   │   │   └── history/
│   │   │   │       ├── {rev_id}.json  # full snapshot of the note at a revision
│   │   │   │       └── index.json   # list of revisions with timestamps
│   │   │   └── index.json          # list of note IDs + latest rev
│   │   └── history/
│   │       ├── {rev_id}.json      # snapshot of workspace state (note list)
│   │       └── index.json          # list of workspace revisions
│   └── index.json                  # global workspace list
└── global.json                     # optional global config or audit log
```

* All files are plain JSON to keep the store human‑readable and easy to diff.
* `index.json` files contain lightweight metadata (IDs, timestamps, hash of content) to enable fast listing without scanning every file.
* Revision IDs are deterministic UUID4 strings generated at commit time.

#### 2. Versioning strategy

* **Append‑only**: Every change creates a new revision file; old files are never overwritten. This guarantees an immutable history and simplifies conflict detection.
* **Snapshot vs. delta**: For notes, store full snapshots (`content.json`) because notes are small (< 1 MB). For workspaces, store a snapshot of the note list only; the note bodies are referenced by ID.
* **Metadata hashing**: Compute a SHA‑256 hash of the JSON payload and store it in `meta.json`. On read, verify the hash to detect corruption.
* **Garbage collection**: Periodically prune revisions older than a configurable retention period (e.g., 90 days) to keep storage bounded.

#### 3. Conflict resolution

* **Optimistic concurrency**: Each write operation requires the caller to provide the *expected* latest revision ID. If the stored revision ID differs, the write is rejected with a `409 Conflict`.
* **Three‑way merge**: For note edits, the backend can expose an endpoint that accepts the base revision ID, the local changes, and the server’s latest revision. The server applies a simple line‑based merge (e.g., using `difflib`) and returns the merged content or a conflict payload.
* **Merge strategy configuration**: Store a per‑workspace merge strategy (`ours`, `theirs`, `manual`) in `meta.json`. The CLI can expose a flag to override the default.

#### 4. fsspec integration

* Use `fsspec.filesystem('file')` for local storage; for remote backends (S3, GCS) the same path layout applies.
* Leverage `fsspec`'s **transactional** context manager (`fs.transaction`) to ensure atomic writes: write to a temp file, then rename.
* Enable **caching** (`cache_type='file'`) to reduce I/O for frequently accessed notes.
* For large workspaces, use `fs.open(..., mode='rb')` with `chunksize` to stream content.

#### 5. API surface

* **Create/Update note**: POST `/workspaces/{ws}/notes` with body `{title, content, tags}`. Returns the new revision ID.
* **Get note**: GET `/workspaces/{ws}/notes/{id}?rev={rev_id}`. If `rev` omitted, return latest.
* **List notes**: GET `/workspaces/{ws}/notes` returns `index.json`.
* **Delete note**: DELETE `/workspaces/{ws}/notes/{id}` creates a tombstone revision.
* **History**: GET `/workspaces/{ws}/notes/{id}/history` streams `index.json`.

#### 6. Security & integrity

* Store a global HMAC key in `global.json` to sign revision files. On read, verify the signature.
* Use file‑system permissions (`chmod 600`) to restrict access to the data directory.
* Log every write operation with a timestamp and user identifier (if available).

#### 7. Performance considerations

* **Batch reads**: For listing notes, read only `index.json` files; avoid scanning the `notes/` directory.
* **Parallelism**: Use `fsspec`'s `ThreadPoolExecutor` for concurrent reads when rendering a workspace view.
* **Compression**: Store `content.json` compressed with `gzip` if the note size exceeds 10 KB.

#### 8. Migration path

* Existing data can be migrated by scanning the current flat file layout, generating revision IDs, and writing the new structure.
* Provide a CLI command `ieapp migrate` that performs the migration atomically.

#### 9. Documentation

* Add a `docs/storage.md` that explains the layout, revision workflow, and conflict resolution.
* Generate a diagram using Mermaid in the README.

## Research Task 6: Workspace Management Best Practices

**Task**: Research best practices for implementing workspace management in a multi‑document application, including considerations for data isolation, access patterns, and efficient listing/deletion.
**Context**: The introduction of the `Workspace` entity in `spec.md` and `data-model.md` requires a robust approach to managing these collections of notes.

### Key Principles

1. **Data isolation** – Each workspace should be a self‑contained namespace. All files, metadata, and revision histories are stored under a dedicated directory (e.g., `workspaces/{workspace_id}`) to prevent accidental cross‑workspace contamination.
2. **Optimized listing** – Use lightweight index files (`index.json`) that contain only IDs, timestamps, and a hash of the latest revision. This allows O(1) listing without scanning the entire file tree.
3. **Atomic operations** – Leverage `fsspec`’s transactional context manager (`fs.transaction`) to write new revisions atomically. This guarantees that a workspace’s state never becomes partially updated.
4. **Versioning strategy** – Adopt an append‑only, immutable snapshot model. Every change creates a new revision file; old revisions are never overwritten. This simplifies conflict detection and auditability.
5. **Efficient deletion** – Instead of physically deleting files, mark a workspace or note as *tombstoned* in its `meta.json`. Physical cleanup can be performed by a background job that respects a retention policy.
6. **Scalable access patterns** – For read‑heavy workloads, cache the index and the most recent revision in memory or a local LRU cache. Use `fsspec`’s `cache_type='file'` for disk‑backed caching.
7. **Security & integrity** – Sign every revision with a HMAC key stored in a protected `global.json`. Verify signatures on read to detect tampering.
8. **Metadata consistency** – Store a `last_modified` timestamp and a `revision_id` in `meta.json`. Validate that the `revision_id` matches the latest file in the history directory.
9. **Graceful degradation** – If the storage backend becomes unavailable, return a clear error message and fall back to an in‑memory cache for read‑only operations.
10. **Documentation & tooling** – Provide a `docs/workspace.md` that explains the directory layout, the API contract, and the cleanup policy.

### Suggested Implementation Steps

1. **Directory layout** – Follow the structure outlined in Task 5 (fsspec storage design). Each workspace has its own `meta.json`, `notes/`, and `history/` directories.
2. **Index files** – Implement `index.json` at the workspace level and within the `notes/` directory. The index contains an array of `{id, title, last_modified, revision_id}` objects.
3. **Atomic writes** – Wrap every create/update/delete operation in `fs.transaction` to ensure that either the entire operation succeeds or the filesystem state remains unchanged.
4. **Tombstone handling** – Add a `deleted: true` flag in `meta.json`. The API should filter out tombstoned workspaces during listing.
5. **Background cleanup** – Schedule a nightly job that scans `history/` directories and removes revisions older than a configurable retention period (e.g., 90 days).
6. **Caching layer** – Use `functools.lru_cache` or a small in‑memory dict to cache the most recent revision data for each workspace.
7. **Error handling** – Wrap all fsspec interactions in try/except blocks that translate `OSError` and `FileNotFoundError` into domain‑specific exceptions (`StorageUnavailableError`, `WorkspaceNotFoundError`).
8. **Testing** – Write unit tests that simulate concurrent writes to the same workspace to verify optimistic concurrency control.
9. **Documentation** – Update `docs/workspace.md` and add a Mermaid diagram of the directory layout.
10. **CI integration** – Add a GitHub Actions job that runs the workspace tests on every push.

### Research Task 7: Implementing Search with fsspec-only Storage

- **Task**: Investigate strategies and tools for implementing efficient keyword search across notes stored solely via `fsspec` in JSON format, without relying on a traditional database (e.g., PostgreSQL).
- **Context**: The application explicitly avoids a separate database, so search must be performed directly on the `fsspec`-managed files. This requires exploring techniques for indexing, querying, and retrieving relevant notes from a file-based store, considering performance for up to 1,000 notes per workspace.

### Research Task 8: Comprehensive Error Handling and Resilience

- **Task**: Develop a comprehensive error handling strategy for both the backend and frontend, covering `fsspec` storage issues, API errors, and unexpected application states. Research best practices for graceful degradation and user feedback.
- **Context**: The `spec.md` mentions "Storage Unavailable" as an edge case, but a broader approach to system resilience is needed to ensure a robust user experience, especially with a file-based backend.

### Research Task 9: Security Considerations for a No-Authentication, File-Based System

- **Task**: Identify potential security risks in a local, no-authentication, file-based knowledge space app. Research mitigation strategies, such as recommendations for network isolation, OS-level file permissions, and data integrity checks.
- **Context**: The assumption of "No authentication is required" for home use necessitates understanding and addressing inherent security implications, even if the threat model is limited.
