<script lang="ts">
  import { play } from '$lib/stores/player';
  import { auth } from '$lib/stores/auth';
  import { tt } from '$lib/i18n';
  import { apiFetch, addTrackToPlaylist, getTrack } from '$lib/api/client';
  import { trackStatuses, monitorTrack, unmonitorTrack } from '$lib/stores/trackStatus';
  import LicenseBadge from '$lib/components/LicenseBadge.svelte';
  import EventList from '$lib/components/EventList.svelte';
  import FavoriteButton from '$lib/components/FavoriteButton.svelte';
  import CommentForm from '$lib/components/CommentForm.svelte';
  import type { PlaylistSummary, EventItem } from '$lib/types/index.js';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
  let track = $derived(data.track);
  let refreshedEvents = $state<EventItem[] | null>(null);
  let events = $derived(refreshedEvents ?? data.track.events ?? []);
  let isOwner = $derived($auth.logged_in && $auth.uid === track.uid);

  let ready = $derived($trackStatuses[track.tid]?.ready ?? true);

  $effect(() => {
    monitorTrack(track.tid);
    return () => unmonitorTrack(track.tid);
  });

  let myPlaylists = $state<PlaylistSummary[]>([]);
  let showPlaylistMenu = $state(false);
  let playlistMsg = $state('');
  let playlistsLoaded = $state(false);

  $effect(() => {
    if ($auth.logged_in && !playlistsLoaded) {
      playlistsLoaded = true;
      apiFetch<{ playlists: PlaylistSummary[] }>(`/user/${$auth.uid}`).then(u => {
        myPlaylists = u.playlists ?? [];
      }).catch(() => {});
    }
  });

  async function handleAddToPlaylist(playlistId: number) {
    try {
      await addTrackToPlaylist(playlistId, track.tid);
      playlistMsg = 'Added';
      showPlaylistMenu = false;
      setTimeout(() => playlistMsg = '', 2000);
    } catch { playlistMsg = 'Failed'; }
  }

  async function refreshComments() {
    try {
      const updated = await getTrack(track.tid);
      refreshedEvents = updated.events ?? [];
    } catch {}
  }

  function handlePlay() {
    if (!ready) return;
    play(track);
  }

  const formats = [
    { label: 'MP3', path: 'mp3' },
    { label: 'Opus', path: 'opus' },
    { label: 'OGG Vorbis', path: 'vorbis' },
    { label: 'AAC', path: 'aac' },
    { label: 'Original', path: 'original' }
  ];
</script>

<svelte:head>
  <title>{track.title} — {$tt('site_name')}</title>
</svelte:head>

<!-- Hero: centered in the starting viewport -->
<div class="min-h-[100vh] flex items-center justify-center -mt-6 -mb-6 relative">
  <article class="flex flex-col gap-6 w-full max-w-2xl">
    <!-- Header -->
    <div class="flex gap-6 items-start">
      <div class="flex-1 min-w-0">
        <h1 class="text-3xl font-extrabold leading-tight text-ellipsis truncate">{track.title}</h1>
        <a href="/user/{track.uid}" class="text-lg text-primary hover:underline">{track.username}</a>
        <span class="text-sm opacity-50 ml-2">{track.date}</span>

        <div class="flex gap-3 mt-4 flex-wrap items-center">
          <button class="btn btn-md" class:btn-primary={ready} class:cursor-not-allowed={!ready} onclick={handlePlay} disabled={!ready} style={!ready ? 'opacity: 0.7;' : ''}>
            {ready ? $tt('player_play') : $tt('track_transcoding')}
          </button>
          {#if $auth.logged_in && !isOwner}
            <FavoriteButton tid={track.tid} />
          {/if}
          {#if $auth.logged_in}
            <div class="relative">
              <button class="btn btn-sm btn-ghost" onclick={() => showPlaylistMenu = !showPlaylistMenu}>+ Playlist</button>
              {#if showPlaylistMenu}
                <div class="absolute top-full left-0 mt-1 bg-base-200 border border-base-300 rounded shadow-lg z-10 min-w-48">
                  {#if myPlaylists.length > 0}
                    {#each myPlaylists as pl}
                      <button class="block w-full text-left px-3 py-1.5 text-sm hover:bg-base-300" onclick={() => handleAddToPlaylist(pl.playlist_id)}>
                        {pl.playlist_name}
                      </button>
                    {/each}
                  {:else}
                    <span class="block px-3 py-1.5 text-sm opacity-60">No playlists</span>
                  {/if}
                  <a href="/playlist/new" class="block px-3 py-1.5 text-sm text-primary hover:bg-base-300 border-t border-base-300">Create new…</a>
                </div>
              {/if}
              {#if playlistMsg}<span class="text-xs ml-1 text-success">{playlistMsg}</span>{/if}
            </div>
          {/if}
          {#if isOwner}
            <a href="/track/{track.tid}/edit" class="btn btn-sm btn-ghost">Edit</a>
          {/if}
        </div>
      </div>
      {#if track.has_art}
        <img src="/api/v1/track/{track.tid}/art" alt="{track.title} cover art" class="w-44 h-44 rounded object-cover shrink-0 shadow-lg" />
      {/if}
    </div>

    <!-- Notes -->
    {#if track.has_notes}
      <div class="prose max-w-none">{@html track.notes_html}</div>
    {/if}

    <!-- Tags -->
    {#if track.has_tags}
      <div class="flex gap-2 flex-wrap items-center">
        <span class="text-sm font-semibold">{$tt('track_tags')}:</span>
        {#each track.tags as tag}
          <a href="/tracks/tag/{tag}" class="badge badge-outline hover:badge-primary">{tag}</a>
        {/each}
      </div>
    {/if}

  </article>

  <!-- Scroll hint -->
  <div class="absolute bottom-6 left-1/2 -translate-x-1/2 opacity-30 animate-bounce">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  </div>
</div>

<!-- Below the fold -->
<div class="max-w-2xl mx-auto">
  <!-- License + Downloads -->
  <section class="py-6 border-t border-base-300 flex flex-wrap items-center gap-4">
    {#if track.has_license}
      <LicenseBadge license={track.license} licenseKey={track.license_key} />
    {/if}
    {#each formats as fmt}
      <a href="/api/v1/track/{track.tid}/{fmt.path}" class="btn btn-sm btn-ghost" download>{fmt.label}</a>
    {/each}
  </section>

  <!-- Comments -->
  <section class="py-8 border-t border-base-300">
    <h3 class="text-lg font-bold mb-4">{$tt('track_comments')}</h3>
    {#if events.length > 0}
      <div class="mb-6">
        <EventList {events} />
      </div>
    {/if}
    <CommentForm target="track" id={track.tid} onCommentPosted={refreshComments} />
  </section>
</div>
