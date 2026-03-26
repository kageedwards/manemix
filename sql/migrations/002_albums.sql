-- Add is_album flag to playlists (albums are playlists where only the author's tracks are allowed)
ALTER TABLE playlists ADD COLUMN IF NOT EXISTS is_album boolean NOT NULL DEFAULT false;
CREATE INDEX IF NOT EXISTS plist_album_idx ON playlists(is_album) WHERE is_album = true;
