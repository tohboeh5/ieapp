# IE-app

## Overview

This repository is a sample project for building a small, home‑use knowledge‑management app. 

## Directory structure

```
├─ frontend/   # Frontend (Bun + SolidStart)
│   ├─ src/
│   ├─ public/
│   └─ bun.lockb
├─ backend/    # Backend (Python + FastAPI, storage: ffspec)
│   ├─ src/
│   ├─ requirements.txt
└─ docker-compose.yml
└─ README.md
```

- **frontend**: UI layer. Built with **Bun** and **SolidStart**. It talks to the backend via HTTP.
- **backend**: API layer. Implemented in **Python** using **FastAPI**. Data persistence is handled by **ffspec** (file‑system‑based storage). #TODO: decide on the exact storage schema and API endpoints.
- **docker-compose.yml**: Orchestrates the two services. #TODO: fill in the service definitions.

## Development

1. **Clone the repo**
   ```bash
   git clone https://github.com/your-username/ieapp.git
   cd ieapp
   ```

2. **Docker Compose** (optional)
   ```bash
   docker compose up
   ```
   This will start both services according to the configuration in `docker-compose.yml`.

## Deployment

- Frontend can be deployed to any static hosting (Vercel, Netlify, etc.).
- Backend can be deployed to a container‑based platform (Heroku, Render, Railway, etc.) or run directly with `uvicorn`.

## Usage

- Add pages/components under `frontend/src/pages` and update routing.
- Add API endpoints under `backend/src/routes` and expose them via FastAPI.

## Known Issues

- Authentication is not implemented yet. #TODO: add JWT/OAuth support.
- Production‑ready database is not set up. #TODO: consider PostgreSQL or other DB.

## Contributing

Feel free to open issues or pull requests. Please create an issue first to discuss major changes.
