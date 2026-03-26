#!/usr/bin/env bash
# Run all SQL migrations in order against the database.
# Usage: ./sql/migrate.sh (from inside the db container)
set -euo pipefail

DB_USER="${PGUSER:-postgres}"
DB_NAME="${POSTGRES_DB:-manemix}"
MIGRATIONS_DIR="$(dirname "$0")/migrations"

if [ ! -d "$MIGRATIONS_DIR" ]; then
  echo "No migrations directory found at $MIGRATIONS_DIR"
  exit 0
fi

for f in "$MIGRATIONS_DIR"/*.sql; do
  [ -f "$f" ] || continue
  echo "Applying $(basename "$f")..."
  psql -U "$DB_USER" -d "$DB_NAME" -f "$f"
done

echo "All migrations applied."
