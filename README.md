# Mane Mix

A music hosting and sharing platform derived from [EqBeats](https://github.com/eqbeats/eqbeats). Artists can upload tracks, organize them into albums and playlists, and listeners can stream, favorite, comment, and follow.

The original C/C++ backend has been completely ported to modern Rust, along with a new frontend built on Svelte.

![Rust Build](https://github.com/kageedwards/manemix/actions/workflows/rust.yml/badge.svg)

## Tech Stack

- **Backend:** Rust (Axum), PostgreSQL, Redis
- **Legacy frontend:** Server-rendered HTML via Tera templates + Nginx
- **SPA frontend:** SvelteKit 5, Tailwind CSS 4, DaisyUI 5
- **Audio processing:** FFmpeg (transcode to MP3, Vorbis, AAC, Opus)
- **Deployment:** Docker Compose, Let's Encrypt via Certbot

## Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [just](https://github.com/casey/just) command runner (optional but recommended)

## Quick Start

```bash
# Start the dev environment (backend / legacy frontend)
just start

# Or without just:
docker compose up --build
```

The site will be available at **http://localhost:8642**.

###  New Frontend (development environment)

```bash
# In another terminal, start the SPA dev server
cd spa && npm install && npm run dev
```

The frontend will be served at **http://localhost:5173** with API requests proxied to the backend.

## Project Structure

```
src/            Rust backend (Axum web server, models, pages/routes)
spa/            SvelteKit SPA frontend
sql/            Database schema and migrations
templates/      Tera HTML templates (legacy frontend)
static/         Static assets (CSS, JS, icons)
conf/nginx/     Nginx configs for dev, prod, and SPA modes
tools/          Transcode script (FFmpeg)
```

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `MANEMIX_POSTGRES` | `postgres://localhost/manemix` | PostgreSQL connection URL |
| `MANEMIX_REDIS` | `redis://127.0.0.1/` | Redis connection URL |
| `MANEMIX_DIR` | `/var/lib/manemix` | Data directory (tracks, art, tmp) |
| `MANEMIX_URL` | `http://localhost:8642` | Public base URL |
| `MANEMIX_BIND` | `0.0.0.0:8100` | Backend listen address |
| `MANEMIX_TEMPLATES` | `templates/**/*` | Tera template glob |
| `MANEMIX_SPA_ORIGIN` | `http://localhost:5173` | Allowed CORS origin for SPA |

## Useful Commands

```bash
just start          # Start dev environment
just stop         # Stop all services
just logs         # Tail logs
just psql         # Open a psql shell
just migrate      # Run all SQL migrations
just migrate 001_create_some_table.sql  # Run a single migration
just redis-cli    # Open a redis-cli shell
just rebuild      # Rebuild without cache
```

## Production Deployment

1. Copy `.env.prod.example` to `.env.prod` and fill in `DOMAIN` and `POSTGRES_PASSWORD`.
2. Bootstrap TLS:
   ```bash
   just bootstrap your-domain.com
   ```
3. Start production:
   ```bash
   just prod
   ```
4. To use the legacy frontend in production, set `FRONTEND=legacy` in `.env.prod`.

Renew certificates with `just cert-renew`.

## API

The backend exposes a JSON API under `/api/v1/` for the SPA frontend. Key endpoints:

- `GET /api/v1/tracks/latest` — latest tracks
- `GET /api/v1/tracks/featured` — featured tracks
- `GET /api/v1/track/:tid` — track details
- `GET /api/v1/user/:uid` — user profile
- `GET /api/v1/artists` — artist listing
- `POST /api/v1/login` / `POST /api/v1/register` — authentication
- `GET /api/v1/me` — current session

Full route list is in `src/pages/mod.rs`.

## License

AGPL-3.0-or-later. See [LICENSE](LICENSE) for details.

Derived from EqBeats, originally licensed under BSD 3-Clause. See [UPSTREAM_LICENSE](UPSTREAM_LICENSE). Not associated with or endorsed by the original developers.
