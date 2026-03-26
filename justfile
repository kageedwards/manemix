# Default: list available recipes
default:
    @just --list

prod_compose := "-f docker-compose.yml -f docker-compose.prod.yml --env-file .env.prod"
bootstrap_compose := "-f docker-compose.yml -f docker-compose.bootstrap.yml --env-file .env.prod"
spa_compose := "-f docker-compose.yml -f docker-compose.spa.yml"
spa_prod_compose := "-f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.spa.yml --env-file .env.prod"

db_user := env("PGUSER", "postgres")
db_name := env("POSTGRES_DB", "manemix")

alias start := dev
alias stop := down

# Start dev environment
dev:
    docker compose up --build

# Start dev environment (detached)
dev-d:
    docker compose up --build -d

# Start SPA dev: backend docker + SPA vite dev server instructions
dev-spa:
    @echo "Starting backend services..."
    docker compose up --build -d
    @echo ""
    @echo "Backend running on http://localhost:8642"
    @echo "Now start the SPA dev server in another terminal:"
    @echo "  cd spa && npm run dev"
    @echo ""
    @echo "SPA will be at http://localhost:5173 with API proxy to :8642"

# Stop all services
down:
    docker compose down

# Tail logs
logs:
    docker compose logs -f

# Rebuild without cache
rebuild:
    docker compose build --no-cache

# Open a psql shell
psql:
    docker compose exec db psql -U {{db_user}} {{db_name}}

# Run all SQL migrations (or a single one: just migrate 002_albums.sql)
migrate file="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "{{file}}" ]; then
        if [ ! -f "sql/migrations/{{file}}" ]; then
            echo "Migration not found: {{file}}"
            exit 1
        fi
        docker compose exec db psql -U {{db_user}} -d {{db_name}} -f "/sql/migrations/{{file}}"
    else
        docker compose exec db bash /sql/migrate.sh
    fi

# Open a redis-cli shell
redis-cli:
    docker compose exec redis redis-cli

# --- Production ---

# Start prod environment (detached) — respects FRONTEND env var
prod:
    #!/usr/bin/env bash
    set -euo pipefail
    FRONTEND="${FRONTEND:-legacy}"
    if [ "$FRONTEND" = "legacy" ]; then
        docker compose {{ prod_compose }} up --build -d
    else
        docker compose {{ spa_prod_compose }} up --build -d
    fi

# Stop prod environment
prod-down:
    #!/usr/bin/env bash
    set -euo pipefail
    FRONTEND="${FRONTEND:-legacy}"
    if [ "$FRONTEND" = "legacy" ]; then
        docker compose {{ prod_compose }} down
    else
        docker compose {{ spa_prod_compose }} down
    fi

# Tail prod logs
prod-logs:
    docker compose {{ prod_compose }} logs -f

# First-time setup: HTTP-only nginx → obtain cert → switch to HTTPS
bootstrap domain:
    @echo "Starting HTTP-only nginx for ACME challenge..."
    docker compose {{ bootstrap_compose }} up -d web
    @echo "Requesting certificate for {{ domain }}..."
    docker compose {{ bootstrap_compose }} run --rm certbot certonly \
        --webroot -w /var/www/certbot \
        -d {{ domain }} --agree-tos --non-interactive \
        --email admin@{{ domain }}
    @echo "Stopping bootstrap nginx..."
    docker compose {{ bootstrap_compose }} down
    @echo "Starting full prod environment with HTTPS..."
    docker compose {{ prod_compose }} up --build -d
    @echo "Done — {{ domain }} should now be live on HTTPS."

# Renew Let's Encrypt certificates
cert-renew:
    docker compose {{ prod_compose }} run --rm certbot renew
    docker compose {{ prod_compose }} exec web nginx -s reload
