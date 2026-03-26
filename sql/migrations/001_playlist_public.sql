-- Add public visibility toggle to playlists (default private)
ALTER TABLE playlists ADD COLUMN IF NOT EXISTS public boolean NOT NULL DEFAULT false;
CREATE INDEX IF NOT EXISTS plist_public_idx ON playlists(public) WHERE public = true;
