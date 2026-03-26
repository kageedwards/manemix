<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth } from '$lib/stores/auth';
  import { play } from '$lib/stores/player';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';
  import type { PlaybackContext } from '$lib/types/index.js';

  let { data } = $props<{ data: PageData }>();
  let album = $derived(data.data.playlist);
  let tracks = $derived(data.data.tracks);
  let isOwner = $derived($auth.logged_in && $auth.uid === album.uid);
  let albumContext: PlaybackContext = $derived({ context: 'playlist', param: String(album.playlist_id) });

  function playAll() {
    if (tracks.length > 0) {
      play(tracks[0], tracks, albumContext);
    }
  }
</script>

<svelte:head>
  <title>{album.playlist_name} — {$tt('site_name')}</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <div class="flex items-center gap-4">
    <div class="flex-1">
      <h1 class="text-2xl font-bold">{album.playlist_name}</h1>
      <a href="/user/{album.uid}" class="text-primary hover:underline text-sm">{album.username}</a>
      <span class="text-sm opacity-60 ml-2">{album.playlist_track_count} tracks</span>
      {#if !album.is_public}
        <span class="badge badge-sm badge-warning ml-2">Draft</span>
      {/if}
    </div>
    <div class="shrink-0 w-20 h-20 rounded bg-base-200 flex items-center justify-center">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16" viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="11" fill="currentColor" opacity="0.25"/>
        <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
        <circle cx="12" cy="12" r="8" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
        <circle cx="12" cy="12" r="6.5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
        <circle cx="12" cy="12" r="5" fill="none" stroke="currentColor" stroke-width="0.3" opacity="0.15"/>
        <circle cx="12" cy="12" r="3.5" fill="currentColor" opacity="0.4"/>
        <circle cx="12" cy="12" r="0.8" fill="var(--color-base-300, #2a2438)"/>
      </svg>
    </div>
  </div>

  {#if album.has_description}
    <div class="prose prose-sm max-w-none">{@html album.description_html}</div>
  {/if}

  <div class="flex gap-2">
    {#if tracks.length > 0}
      <button class="btn btn-primary btn-sm" onclick={playAll}>Play All</button>
    {/if}
    {#if isOwner}
      <a href="/album/{album.playlist_id}/edit" class="btn btn-ghost btn-sm">Edit Album</a>
    {/if}
  </div>

  {#if tracks.length > 0}
    <TrackList {tracks} playbackContext={albumContext} />
  {:else}
    <p class="opacity-60">No tracks in this album yet.</p>
  {/if}
</div>
