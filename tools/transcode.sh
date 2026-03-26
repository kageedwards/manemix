#!/bin/bash
# Transcode an uploaded audio file into all delivery formats.
# Usage: transcode.sh ID INPUT
# Expects MANEMIX_DIR to be set.

set -e

if [[ "$#" -ne 2 ]]; then
    echo "Usage: $0 ID INPUT" >&2
    exit 1
fi

ID="$1"
INPUT="$2"
BASE="$MANEMIX_DIR/tracks/$ID"
LOG="$MANEMIX_DIR/ffmpeg.log"

# Remove any previous transcoded files for this track
find "$MANEMIX_DIR/tracks" -name "$ID.*" -delete

# Vorbis (first — if this fails, the source is bad)
ffmpeg -loglevel error -probesize 10000000 -y -i "$INPUT" \
    -acodec libvorbis -q:a 4 -vn "$BASE.ogg" 2>&1 >> "$LOG"

# AAC
ffmpeg -loglevel error -probesize 10000000 -y -i "$INPUT" \
    -acodec aac -b:a 128k -vn "$BASE.m4a" 2>&1 >> "$LOG" || true

# Opus
ffmpeg -loglevel error -probesize 10000000 -y -i "$INPUT" \
    -acodec libopus -b:a 128k -vn "$BASE.opus" 2>&1 >> "$LOG" || true

# MP3: if the source is already MP3, keep it as-is; otherwise transcode
if [[ $(file -b --mime-type "$INPUT") = "audio/mpeg" ]]; then
    mv "$INPUT" "$BASE.mp3"
    ln -sf "$ID.mp3" "$BASE.orig.mp3"
else
    ffmpeg -loglevel error -probesize 10000000 -y -i "$INPUT" \
        -acodec libmp3lame -q:a 0 -vn "$BASE.mp3" 2>&1 >> "$LOG"
    EXT="${INPUT##*.}"
    mv "$INPUT" "$BASE.orig.$EXT"
fi

# Update tags — call the Rust binary's updatetags subcommand if available,
# otherwise skip (tags will be set on next rename).
if command -v manemix-updatetags &>/dev/null; then
    manemix-updatetags "$ID" || true
fi
