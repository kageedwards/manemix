<script lang="ts">
  import type { PlaylistSummary } from '$lib/types/index.js';

  interface Props {
    playlist: PlaylistSummary;
  }
  let { playlist }: Props = $props();
</script>

<a href="/playlist/{playlist.playlist_id}" class="card bg-base-200 p-3 hover:bg-base-300 transition-colors">
  <div class="font-semibold text-base-content truncate">{playlist.playlist_name}</div>
  <div class="text-sm opacity-70">
    {playlist.playlist_track_count} tracks
    {#if playlist.username}
      <span class="mx-1">·</span>
      <span>{playlist.username}</span>
    {/if}
  </div>
  {#if playlist.has_description}
    <div class="text-xs opacity-50 mt-1 truncate">{playlist.description}</div>
  {/if}
</a>
