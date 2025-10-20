# Tasks: Knowledge Space App

## Phase 1: Setup

This phase focuses on initial project setup, environment configuration, and critical research tasks that inform subsequent development.

- [ ] T001 Create project structure for `backend/` and `ieapp-cli/` directories at the root, and `frontend/` directory
- [ ] T002 Install uv Python package manager in `backend/` and `ieapp-cli/`
- [ ] T003 Initialize Python environment with `uv` in `ieapp-cli/` as a Python library
- [ ] T004 Initialize Python backend with `uv` in `backend/`, importing `ieapp-cli` to start the API
- [ ] T005 Initialize SolidJS frontend with `package.json` and `bun.lock` in `frontend/`
- [ ] T006 Research testing frameworks for Python (FastAPI) and SolidJS (bunjs) in `specs/001-knowledge-space-app/research.md`
- [ ] T007 Research best practices for Python library CLI interface in `specs/001-knowledge-space-app/research.md`
- [ ] T008 Research applying TDD principles to Python (FastAPI) and SolidJS (bunjs) in `specs/001-knowledge-space-app/research.md`
- [ ] T009 Research integration testing strategies for FastAPI and SolidJS in `specs/001-knowledge-space-app/research.md`
- [ ] T010 Research fsspec storage design for backend-managed versioning and conflict resolution in `specs/001-knowledge-space-app/research.md`
- [ ] T011 Research workspace management best practices in `specs/001-knowledge-space-app/research.md`
- [ ] T012 Research implementing search with fsspec-only storage in `specs/001-knowledge-space-app/research.md`
- [ ] T013 Research comprehensive error handling and resilience in `specs/001-knowledge-space-app/research.md`
- [ ] T014 Research security considerations for a no-authentication, file-based system in `specs/001-knowledge-space-app/research.md`
- [ ] T015 Implement Docker Compose setup for backend and frontend in `docker-compose.yaml`

## Phase 2: Foundational

This phase includes tasks that are prerequisites for all user stories, such as core data storage mechanisms and common utilities.

- [ ] T016 Implement fsspec-based JSON storage utility in `backend/src/utils/fsspec_storage.py`
- [ ] T017 Implement UUID generation utility in `backend/src/utils/uuid_generator.py`

## Phase 3: User Story 1 - Manage Workspaces (P1)

**Story Goal**: As a user, I want to create, list, and delete workspaces so that I can organize my knowledge into distinct, manageable collections.
**Independent Test Criteria**: A user can perform a full CRUD cycle on a workspace and see the changes reflected immediately.

- [ ] T018 [P] [US1] Define Workspace Pydantic model in `backend/src/models/workspace.py`
- [ ] T019 [US1] Implement Workspace service (CRUD) in `backend/src/services/workspace_service.py`
- [ ] T020 [US1] Implement FastAPI endpoints for Workspace (create, list, get, delete) in `backend/src/api/workspace_api.py`
- [ ] T021 [P] [US1] Create Workspace API client in `frontend/src/services/workspace_api_client.ts`
- [ ] T022 [P] [US1] Create Workspace list component in `frontend/src/components/WorkspaceList.tsx`
- [ ] T023 [P] [US1] Create New Workspace form component in `frontend/src/components/NewWorkspaceForm.tsx`
- [ ] T024 [US1] Integrate Workspace components into a page in `frontend/src/pages/Workspaces.tsx`

## Phase 4: User Story 2 - Create, Edit, and Delete Notes (P1)

**Story Goal**: As a user, I want to create, edit, and delete notes so that I can capture and manage information.
**Independent Test Criteria**: A user can perform a full CRUD cycle on a note and see the changes reflected immediately.

- [ ] T025 [P] [US2] Define Note Pydantic model in `backend/src/models/note.py`
- [ ] T026 [US2] Implement Note service (CRUD) in `backend/src/services/note_service.py`
- [ ] T027 [US2] Implement FastAPI endpoints for Note (create, list, get, update, delete) in `backend/src/api/note_api.py`
- [ ] T028 [P] [US2] Create Note API client in `frontend/src/services/note_api_client.ts`
- [ ] T029 [P] [US2] Create Note list component in `frontend/src/components/NoteList.tsx`
- [ ] T030 [P] [US2] Create New Note form component in `frontend/src/components/NewNoteForm.tsx`
- [ ] T031 [P] [US2] Create Note editor component in `frontend/src/components/NoteEditor.tsx`
- [ ] T032 [US2] Integrate Note components into a page in `frontend/src/pages/Notes.tsx`

## Phase 5: User Story 3 - Search Notes (P2)

**Story Goal**: As a user, I want to search notes by keyword so that I can quickly find relevant information.
**Independent Test Criteria**: A user can enter a keyword and receive matching notes within a short time.

- [ ] T033 [US3] Implement search logic in Note service in `backend/src/services/note_service.py`
- [ ] T034 [US3] Implement FastAPI endpoint for note search in `backend/src/api/note_api.py`
- [ ] T035 [P] [US3] Create Search input component in `frontend/src/components/SearchInput.tsx`
- [ ] T036 [US3] Display search results in `frontend/src/pages/Notes.tsx`

## Phase 6: User Story 4 - Export and Import Notes (P3)

**Story Goal**: As a user, I want to export notes to Markdown and import from Markdown so that I can backup or migrate data.
**Independent Test Criteria**: A user can export a note to a Markdown file and later import it back, preserving content.

- [ ] T037 [US4] Implement export to Markdown logic in Note service in `backend/src/services/note_service.py`
- [ ] T038 [US4] Implement import from Markdown logic in Note service in `backend/src/services/note_service.py`
- [ ] T039 [US4] Implement FastAPI endpoints for export/import in `backend/src/api/note_api.py`
- [ ] T040 [P] [US4] Add Export button to Note view in `frontend/src/components/NoteEditor.tsx`
- [ ] T041 [US4] Add Import functionality (file upload) in `frontend/src/components/NewNoteForm.tsx`

## Phase 7: User Story 5 - Show, Search, and Edit Notes with LLM (P4)

**Story Goal**: As a user, I want to show, search, and edit notes using an LLM as an app in the Model Context Protocol (MCP) so that I can interact with my knowledge space more naturally.
**Independent Test Criteria**: A user can interact with the LLM to perform these actions and see the results.

- [ ] T042 [US5] Integrate backend with MCP in `backend/src/mcp_integration.py`
- [ ] T043 [US5] Expose Note and Workspace services via MCP in `backend/src/mcp_integration.py`
- [ ] T044 [P] [US5] Create LLM interaction interface in `frontend/src/components/LLMInterface.tsx`

## Final Phase: Polish & Cross-Cutting Concerns

- [ ] T045 Implement structured logging for CRUD operations in `backend/src/utils/logger.py`
- [ ] T046 Implement comprehensive error handling and resilience across backend and frontend
- [ ] T047 Address security considerations (e.g., file permissions, data integrity)
- [ ] T048 Optimize performance for SC-001, SC-002, SC-003, SC-004, SC-006, SC-007
- [ ] T049 Update `README.md` with detailed setup and usage instructions
- [ ] T050 Update `quickstart.md` with detailed setup and usage instructions
- [ ] T051 Add CI/CD configuration for automated testing and deployment

## Dependencies

- Phase 1 (Setup) -> Phase 2 (Foundational)
- Phase 2 (Foundational) -> Phase 3 (US1)
- Phase 2 (Foundational) -> Phase 4 (US2)
- Phase 3 (US1) -> Phase 4 (US2) (Notes depend on Workspaces)
- Phase 4 (US2) -> Phase 5 (US3)
- Phase 4 (US2) -> Phase 6 (US4)
- Phase 4 (US2) -> Phase 7 (US5)
- Phase 7 (US5) can be developed in parallel with other UI/API tasks once core Note/Workspace functionality is stable.
- Final Phase (Polish & Cross-Cutting Concerns) can run in parallel with all other phases, but many tasks depend on core functionality being present.

## Parallel Execution Examples

- **User Story 1 (Manage Workspaces)**:
    - Backend: T018, T019, T020 can be developed sequentially.
    - Frontend: T021, T022, T023 can be developed in parallel. T024 integrates them.
- **User Story 2 (Create, Edit, and Delete Notes)**:
    - Backend: T025, T026, T027 can be developed sequentially.
    - Frontend: T028, T029, T030, T031 can be developed in parallel. T032 integrates them.

## Implementation Strategy

The implementation will follow an MVP-first approach, delivering incremental value. User Story 1 (Manage Workspaces) and User Story 2 (Create, Edit, and Delete Notes) are P1 and form the core MVP. Subsequent user stories will be integrated in priority order. Each user story will be developed with its backend API and frontend UI components, ensuring independent testability.

## Report

- **Total task count**: 51
- **Task count per user story**:
    - Setup: 14
    - Foundational: 2
    - US1 (Manage Workspaces): 7
    - US2 (Create, Edit, and Delete Notes): 8
    - US3 (Search Notes): 4
    - US4 (Export and Import Notes): 5
    - US5 (Show, Search, and Edit Notes with LLM): 3
    - Polish & Cross-Cutting Concerns: 7
- **Parallel opportunities identified**: Many tasks within frontend development for each user story, and some backend tasks (e.g., defining models) can be done in parallel. Research tasks can also be done in parallel.
- **Independent test criteria for each story**: Clearly defined in each user story phase.
- **Suggested MVP scope**: User Story 1 (Manage Workspaces) and User Story 2 (Create, Edit, and Delete Notes).
- **Format validation**: All tasks follow the checklist format: `- [ ] [TaskID] [P?] [Story?] Description with file path`.
