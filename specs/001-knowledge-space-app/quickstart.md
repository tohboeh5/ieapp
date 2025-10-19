# Quickstart Guide: Knowledge Space App

This guide provides instructions to quickly set up and run the Knowledge Space App, including both the backend API and the frontend application.

## 1. Prerequisites

Before you begin, ensure you have the following installed:

- **Python 3.9+**: For the backend (if running directly).
- **bun**: For the frontend (if running directly).
- **Git**: For cloning the repository.
- **Docker and Docker Compose**: (Optional) For containerized development environment.

## 2. Clone the Repository

First, clone the project repository to your local machine:

```bash
git clone https://github.com/tohboeh5/ieapp.git
cd ieapp
```

## 3. Running with Docker Compose (Recommended for Development)

For a fully containerized development environment, you can use Docker Compose. This will start both the backend API and the frontend application.

From the project root directory, run:

```bash
docker compose up --build
```

Once the services are up, the API will be accessible at `http://localhost:8000` and the frontend at `http://localhost:3000` (or other configured ports).

## 4. Backend Setup and Run (Manual)

If you prefer to run the backend directly without Docker:

### 4.1. Setup

Navigate to the `backend` directory and install the Python dependencies:

```bash
cd backend
pip install -r requirements.txt
```

### 4.2. Configuration (Optional)

The backend uses `fsspec` and can be configured to use different storage backends (e.g., local, S3, MinIO, Azure Blob). By default, it uses local storage.

You can specify the storage location using an environment variable or a configuration file. For local storage, you might set:

```bash
export FSSPEC_STORAGE_PATH="./data"
```

### 4.3. Run the Backend API

To start the FastAPI server:

```bash
uvicorn main:app --reload
```

The API will be accessible at `http://127.0.0.1:8000`.

## 5. Frontend Setup and Run (Manual)

If you prefer to run the frontend directly without Docker:

### 5.1. Setup

Navigate to the `frontend` directory and install the dependencies using bun:

```bash
cd frontend
bun install
```

### 5.2. Run the Frontend Application

To start the SolidJS development server:

```bash
bun run dev
```

The frontend application will typically be available at `http://localhost:3000` (or another port if 3000 is in use).

## 6. Using the Application

Once both the backend and frontend are running (either via Docker Compose or manually):

- Open your web browser and navigate to the frontend URL (e.g., `http://localhost:3000`).
- You can now create, list, and delete workspaces. Within each workspace, you can create, edit, delete, and search for notes through the user interface.

## 7. Backend as a Library

If you wish to use the backend as a Python library in your own projects, you can install it:

```bash
pip install ./backend
```

Then, you can import and use its modules:

```python
from knowledge_space.api import WorkspaceAPI, NoteAPI
# ... use the library
```

Further details on library usage and CLI will be provided in the `research.md` and `plan.md` updates.
