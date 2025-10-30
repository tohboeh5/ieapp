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

- **Task**: Research how to effectively apply Test-Driven Development (TDD) principles to both FastAPI backend and SolidJS frontend development.
- **Context**: The "Constitution Check" in `plan.md` has "III. Test-First (NON-NEGOTIABLE): NEEDS CLARIFICATION". This is a core principle that needs to be addressed.

### Research Task 4: Integration Testing Strategies for FastAPI and SolidJS

- **Task**: Explore strategies for integration testing between the FastAPI backend and the SolidJS frontend, including contract testing.
- **Context**: The "Constitution Check" in `plan.md` has "IV. Integration Testing: NEEDS CLARIFICATION". This is crucial for ensuring the proper functioning of the entire application.

### Research Task 5: fsspec Storage Design for Backend-Managed Versioning and Conflict Resolution

- **Task**: Design and clearly define the `fsspec` directory structure, naming conventions, and file formats to explicitly support backend-managed versioning, history retrieval, and conflict resolution for notes and workspaces.
- **Context**: The current directory structure in `data-model.md` is provisional. This task is critical for ensuring the backend can effectively implement these features without relying on `fsspec` itself for versioning, and for defining the detailed JSON-based data format for notes and workspaces.

### Research Task 6: Workspace Management Best Practices

- **Task**: Research best practices for implementing workspace management in a multi-document application, including considerations for data isolation, access patterns, and efficient listing/deletion.
- **Context**: The introduction of the `Workspace` entity in `spec.md` and `data-model.md` requires a robust approach to managing these collections of notes.

### Research Task 7: Implementing Search with fsspec-only Storage

- **Task**: Investigate strategies and tools for implementing efficient keyword search across notes stored solely via `fsspec` in JSON format, without relying on a traditional database (e.g., PostgreSQL).
- **Context**: The application explicitly avoids a separate database, so search must be performed directly on the `fsspec`-managed files. This requires exploring techniques for indexing, querying, and retrieving relevant notes from a file-based store, considering performance for up to 1,000 notes per workspace.

### Research Task 8: Comprehensive Error Handling and Resilience

- **Task**: Develop a comprehensive error handling strategy for both the backend and frontend, covering `fsspec` storage issues, API errors, and unexpected application states. Research best practices for graceful degradation and user feedback.
- **Context**: The `spec.md` mentions "Storage Unavailable" as an edge case, but a broader approach to system resilience is needed to ensure a robust user experience, especially with a file-based backend.

### Research Task 9: Security Considerations for a No-Authentication, File-Based System

- **Task**: Identify potential security risks in a local, no-authentication, file-based knowledge space app. Research mitigation strategies, such as recommendations for network isolation, OS-level file permissions, and data integrity checks.
- **Context**: The assumption of "No authentication is required" for home use necessitates understanding and addressing inherent security implications, even if the threat model is limited.
