# Data Model: Knowledge Space App

## Entity: Workspace

Represents a distinct collection of notes, allowing users to organize their knowledge.

### Attributes:

- `id` (UUID): A unique identifier for the workspace.
  - **Validation**: Must be a valid UUID format.
- `name` (string): The name of the workspace.
  - **Validation**: Required, minimum length 1 character, maximum length 100 characters, unique per user.
- `created_at` (timestamp): The date and time when the workspace was first created.
  - **Validation**: Automatically generated, read-only.
- `updated_at` (timestamp): The date and time when the workspace was last modified.
  - **Validation**: Automatically updated on modification, read-only.

### Relationships:

- **Has Many Notes**: A workspace can contain multiple `Note` entities. The relationship is established via the `workspace_id` attribute in the `Note` entity.

### State Transitions:

- **Created**: A workspace is created with an `id`, `name`, `created_at`, and `updated_at`.
- **Deleted**: An existing workspace can be removed from the system. This action should cascade and delete all associated notes.

## Entity: Note

Represents a knowledge entry within the application.

### Attributes:

- `id` (UUID): A unique identifier for the note.
  - **Validation**: Must be a valid UUID format.
- `workspace_id` (UUID): The ID of the workspace to which the note belongs.
  - **Validation**: Required, must be a valid UUID referencing an existing `Workspace`.
- `title` (string): The title of the note.
  - **Validation**: Required, minimum length 1 character, maximum length 255 characters.
- `content` (string): The main body or content of the note.
  - **Validation**: Required.
- `tags` (list of strings): A list of keywords or categories associated with the note.
  - **Validation**: Optional, each tag string must be alphanumeric and can contain hyphens, maximum length 50 characters per tag.
- `created_at` (timestamp): The date and time when the note was first created.
  - **Validation**: Automatically generated, read-only.
- `updated_at` (timestamp): The date and time when the note was last modified.
  - **Validation**: Automatically updated on modification, read-only.

### Relationships:

- **Belongs To Workspace**: A note belongs to one `Workspace` entity, identified by `workspace_id`.

### State Transitions:

- **Created**: A note is created within a specific `workspace_id` with an `id`, `title`, `content`, `created_at`, and `updated_at`.
- **Edited**: The `title`, `content`, or `tags` of an existing note can be modified, which updates `updated_at`.
- **Deleted**: An existing note can be removed from the system.

## Data Format for fsspec Storage

To enable the backend to implement versioning, history retrieval, and conflict resolution, notes will be stored in a structured format within the `fsspec` backend. The design of the folder tree and naming conventions for data storage is crucial for these backend-driven features. Each note will be stored as a separate file, and a directory structure will be used to organize notes by workspace and facilitate version tracking.

### Directory Structure (Provisional):

```
<fsspec_root>/
├── workspaces/
│   ├── <workspace_id_1>/
│   │   ├── notes/
│   │   │   ├── <note_id_1>.json
│   │   │   ├── <note_id_2>.json
│   │   │   └── ...
│   │   └── workspace_metadata.json
│   ├── <workspace_id_2>/
│   │   └── ...
│   └── ...
└── global_metadata.json
```

### Note File Format (`<note_id>.json`):

Each note will be stored as a JSON file with the following structure:

```json
{
  "id": "[UUID]",
  "workspace_id": "[UUID]",
  "title": "[string]",
  "content": "[string]",
  "tags": ["[string]", ...],
  "created_at": "[timestamp]",
  "updated_at": "[timestamp]",
  "version_history": [
    {
      "timestamp": "[timestamp]",
      "user": "[user_id_or_identifier]", // Future: for multi-user or audit
      "change_summary": "[string]" // Future: brief description of change
    }
  ]
}
```

- **`version_history`**: This array will store metadata about each significant change to the note. Initially, it will track `timestamp`. In the future, `user` and `change_summary` can be added to facilitate detailed history and conflict resolution.

### Workspace Metadata File Format (`workspace_metadata.json`):

Each workspace directory will contain a `workspace_metadata.json` file:

```json
{
  "id": "[UUID]",
  "name": "[string]",
  "created_at": "[timestamp]",
  "updated_at": "[timestamp]"
}
```

### Global Metadata File Format (`global_metadata.json`):

A `global_metadata.json` file at the root will track all workspaces:

```json
{
  "workspaces": [
    {
      "id": "[UUID]",
      "name": "[string]",
      "path": "workspaces/[UUID]"
    }
  ]
}
```

### Conflict Resolution Strategy:

- **Backend-Managed Last-Write-Wins**: For concurrent edits, the backend system will implement a last-write-wins strategy at the file level. The `updated_at` timestamp in the note file will determine the most recent version. Users will be notified of potential conflicts by the backend.
- **Future Enhancements**: The `version_history` field in the note file provides a foundation for more sophisticated conflict resolution mechanisms (e.g., three-way merge) in the future, by allowing the backend to track changes and manage versions.
