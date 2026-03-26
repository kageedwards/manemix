<script lang="ts">
  import { tt } from '$lib/i18n';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import {
    getAlbum, editAlbum, deleteAlbum, publishAlbum, unpublishAlbum,
    removeTrackFromAlbum, addTrackToAlbum, apiFetch
  } from '$lib/api/client';
  import type { AlbumData, Track } from '$lib/types/index.js';

  let aid = $derived(Number($page.params.id));
  let album = $state<AlbumData | null>(null);
  let name = $state('');
  let description = $state('');
  let isPublic = $state(false);
  let error = $state('');
  let message = $state('');
  let saving = $state(false);
  let publishing = $state(false);

  // Track picker state
  let myTracks = $state<Track[]>([]);
  let showTrackPicker = $state(false);

  $effect(() => { loadAlbum(); });

  async function loadAlbum() {
    try {
      album = await getAlbum(aid);
      name = album.playlist.playlist_name;
      description = album.playlist.description;
      isPublic = album.playlist.is_public;
      // Load user's own tracks for the picker
      if ($auth.uid) {
        const profile = await apiFetch<{ tracks: Track[] }>(`/user/${$auth.uid}`);
        myTracks = profile.tracks;
      }
    } catch { error = 'Failed to load album'; }
  }

  async function handleSave(e: Event) {
    e.preventDefault();
    saving = true; error = ''; message = '';
    try {
      await editAlbum(aid, { name, description });
      message = 'Saved';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save';
    } finally { saving = false; }
  }
</script>

<svelte:head>
  <title>Edit Album — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-lg mx-auto">
  <a href="/album/{aid}" class="text-sm text-primary hover:underline mb-4 block">← Back to album</a>

  {#if album}
    <h1 class="text-xl font-bold mb-6">Edit: {album.playlist.playlist_name}</h1>

    <form onsubmit={handleSave} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Name</span>
        <input type="text" bind:value={name} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Description</span>
        <textarea bind:value={description} class="textarea textarea-bordered w-full" rows="3"></textarea>
      </div>

      <button type="submit" class="btn btn-primary btn-sm" disabled={saving}>
        {saving ? 'Saving…' : 'Save Changes'}
      </button>
    </form>

    <!-- Publish / Unpublish -->
    <div class="card bg-base-200 p-4 mt-6">
      {#if isPublic}
        <p class="text-sm mb-2">This album is <span class="text-success font-semibold">published</span>.</p>
        <button class="btn btn-sm btn-warning" disabled={publishing} onclick={async () => {
          publishing = true; error = '';
          try { await unpublishAlbum(aid); isPublic = false; message = 'Album unpublished'; }
          catch { error = 'Failed to unpublish'; }
          finally { publishing = false; }
        }}>Unpublish</button>
      {:else}
        <p class="text-sm mb-2">This album is a <span class="text-warning font-semibold">draft</span>. Publishing will make it public and publish all contained tracks.</p>
        {#if album.tracks.length === 0}
          <p class="text-sm text-warning mb-2">Add at least one track before publishing.</p>
        {/if}
        <button class="btn btn-sm btn-success" disabled={publishing || album.tracks.length === 0} onclick={async () => {
          if (!confirm('Publish this album? All tracks in it will also be published.')) return;
          publishing = true; error = '';
          try { await publishAlbum(aid); album = await getAlbum(aid); isPublic = true; message = 'Album published'; }
          catch { error = 'Failed to publish'; }
          finally { publishing = false; }
        }}>Publish Album</button>
      {/if}
    </div>

    <!-- Tracks -->
    <div class="card bg-base-200 p-4 mt-6">
      <div class="flex items-center justify-between mb-3">
        <span class="text-sm font-semibold">Tracks ({album.tracks.length})</span>
        <button class="btn btn-xs btn-ghost" onclick={() => showTrackPicker = !showTrackPicker}>
          {showTrackPicker ? 'Cancel' : '+ Add Track'}
        </button>
      </div>

      {#if showTrackPicker}
        <div class="mb-4 p-3 bg-base-300 rounded-lg">
          <span class="text-xs font-medium block mb-2">Your tracks:</span>
          {#each myTracks as track (track.tid)}
            {@const alreadyAdded = album.tracks.some(t => t.tid === track.tid)}
            <button
              class="btn btn-xs btn-ghost w-full justify-start text-left mb-1"
              disabled={alreadyAdded}
              onclick={async () => {
                try {
                  await addTrackToAlbum(aid, track.tid);
                  album = await getAlbum(aid);
                } catch { error = 'Failed to add track'; }
              }}
            >
              {track.title} {alreadyAdded ? '(added)' : ''}
            </button>
          {/each}
          {#if myTracks.length === 0}
            <p class="text-xs opacity-60">You have no tracks. Upload some first.</p>
          {/if}
        </div>
      {/if}

      {#if album.tracks.length > 0}
        <div class="flex flex-col gap-1">
          {#each album.tracks as track, i (track.tid)}
            <div class="flex items-center gap-2 text-sm py-1">
              <span class="opacity-50 w-6 text-right">{i + 1}.</span>
              <a href="/track/{track.tid}" class="flex-1 hover:text-primary truncate">{track.title}</a>
              {#if !track.is_visible}
                <span class="badge badge-xs badge-warning">unpublished</span>
              {/if}
              <button class="btn btn-ghost btn-xs text-error" onclick={async () => {
                try {
                  await removeTrackFromAlbum(aid, track.tid);
                  if (album) {
                    album = { ...album, tracks: album.tracks.filter(t => t.tid !== track.tid) };
                  }
                } catch { error = 'Failed to remove track'; }
              }}>×</button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm opacity-60">No tracks in this album. Add your tracks above.</p>
      {/if}
    </div>

    {#if error}<p class="text-error text-sm mt-4">{error}</p>{/if}
    {#if message}<p class="text-success text-sm mt-4">{message}</p>{/if}

    <div class="mt-6 pt-4 border-t border-base-300">
      <button class="btn btn-sm btn-error btn-outline" onclick={async () => {
        if (!confirm('Delete this album? (Tracks will not be deleted)')) return;
        try { await deleteAlbum(aid); goto(`/user/${$auth.uid}`); }
        catch { error = 'Failed to delete'; }
      }}>Delete Album</button>
    </div>
  {:else if error}
    <p class="text-error">{error}</p>
  {:else}
    <p class="opacity-60">Loading…</p>
  {/if}
</div>
