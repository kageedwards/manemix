<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth } from '$lib/stores/auth';
  import { play } from '$lib/stores/player';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';
  import type { PlaybackContext } from '$lib/types/index.js';

  let { data } = $props<{ data: PageData }>();
  let playlist = $derived(data.data.playlist);
  let tracks = $derived(data.data.tracks);
  let isOwner = $derived($auth.logged_in && $auth.uid === playlist.uid);
  let playlistContext: PlaybackContext = $derived({ context: 'playlist', param: String(playlist.playlist_id) });

  function playAll() {
    if (tracks.length > 0) {
      play(tracks[0], tracks, playlistContext);
    }
  }
</script>

<svelte:head>
  <title>{playlist.playlist_name} — {$tt('site_name')}</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <div>
    <h1 class="text-2xl font-bold">{playlist.playlist_name}</h1>
    <a href="/user/{playlist.uid}" class="text-primary hover:underline text-sm">{playlist.username}</a>
    <span class="text-sm opacity-60 ml-2">{playlist.playlist_track_count} tracks</span>
  </div>

  {#if playlist.has_description}
    <div class="prose prose-sm max-w-none">{@html playlist.description_html}</div>
  {/if}

  <div class="flex gap-2">
    <button class="btn btn-sm btn-primary" onclick={playAll}>Play All</button>
    {#if isOwner}
      <a href="/playlist/{playlist.playlist_id}/edit" class="btn btn-sm btn-ghost">Edit Playlist</a>
    {/if}
    {#if !playlist.is_public}
      <span class="badge badge-sm opacity-60">Private</span>
    {/if}
  </div>

  {#if tracks.length > 0}
    <TrackList {tracks} playbackContext={playlistContext} />
  {:else}
    <p class="opacity-60">This playlist is empty.</p>
  {/if}
</div>
