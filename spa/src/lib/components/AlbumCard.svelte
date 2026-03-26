<script lang="ts">
  import type { AlbumSummary } from '$lib/types/index.js';

  interface Props {
    album: AlbumSummary;
  }
  let { album }: Props = $props();
</script>

<a href="/album/{album.playlist_id}" class="card card-side bg-base-200 p-3 gap-3 items-center hover:bg-base-300 transition-colors">
  <div class="shrink-0 w-14 h-14 rounded bg-base-300 flex items-center justify-center">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-11 h-11" viewBox="0 0 24 24" aria-hidden="true">
      <!-- Disc body -->
      <circle cx="12" cy="12" r="11" fill="currentColor" opacity="0.25"/>
      <!-- Grooves -->
      <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
      <circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
      <circle cx="12" cy="12" r="6.5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
      <circle cx="12" cy="12" r="5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
      <!-- Center label -->
      <circle cx="12" cy="12" r="3.5" fill="currentColor" opacity="0.4"/>
      <!-- Spindle hole -->
      <circle cx="12" cy="12" r="0.8" fill="var(--color-base-300, #2a2438)"/>
    </svg>
  </div>
  <div class="flex-1 min-w-0">
    <div class="font-semibold text-base-content truncate">{album.playlist_name}</div>
    <div class="text-sm opacity-70">
      {album.playlist_track_count} tracks
      {#if album.username}
        <span class="mx-1">·</span>
        <span>{album.username}</span>
      {/if}
    </div>
    {#if album.has_description}
      <div class="text-xs opacity-50 mt-1 truncate">{album.description}</div>
    {/if}
  </div>
</a>
