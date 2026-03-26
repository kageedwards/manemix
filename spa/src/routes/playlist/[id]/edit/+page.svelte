<script lang="ts">
  import { tt } from '$lib/i18n';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { apiFetch, getPlaylist, editPlaylist, deletePlaylist, removeTrackFromPlaylist } from '$lib/api/client';
  import type { PlaylistData, Track } from '$lib/types/index.js';

  let pid = $derived(Number($page.params.id));
  let playlist = $state<PlaylistData | null>(null);
  let name = $state('');
  let description = $state('');
  let isPublic = $state(false);
  let error = $state('');
  let message = $state('');
  let saving = $state(false);

  $effect(() => { loadPlaylist(); });

  async function loadPlaylist() {
    try {
      playlist = await getPlaylist(pid);
      name = playlist.playlist.playlist_name;
      description = playlist.playlist.description;
      isPublic = playlist.playlist.is_public;
    } catch { error = 'Failed to load playlist'; }
  }

  async function handleSave(e: Event) {
    e.preventDefault();
    saving = true; error = ''; message = '';
    try {
      await editPlaylist(pid, { name, description, public: isPublic });
      message = 'Saved';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save';
    } finally { saving = false; }
  }

  async function handleRemoveTrack(tid: number) {
    try {
      await removeTrackFromPlaylist(pid, tid);
      if (playlist) {
        playlist = { ...playlist, tracks: playlist.tracks.filter(t => t.tid !== tid) };
      }
    } catch { error = 'Failed to remove track'; }
  }

  async function handleDelete() {
    if (!confirm('Delete this playlist?')) return;
    try {
      await deletePlaylist(pid);
      goto(`/user/${$auth.uid}`);
    } catch { error = 'Failed to delete'; }
  }
</script>

<svelte:head>
  <title>Edit Playlist — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-lg mx-auto">
  <a href="/playlist/{pid}" class="text-sm text-primary hover:underline mb-4 block">← Back to playlist</a>

  {#if playlist}
    <h1 class="text-xl font-bold mb-6">Edit: {playlist.playlist.playlist_name}</h1>

    <form onsubmit={handleSave} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Name</span>
        <input type="text" bind:value={name} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Description</span>
        <textarea bind:value={description} class="textarea textarea-bordered w-full" rows="3"></textarea>
      </div>

      <label class="flex items-center gap-2 cursor-pointer">
        <input type="checkbox" bind:checked={isPublic} class="checkbox checkbox-sm checkbox-primary" />
        <span class="text-sm">Public playlist (visible to everyone)</span>
      </label>

      <button type="submit" class="btn btn-primary btn-sm" disabled={saving}>
        {saving ? 'Saving…' : 'Save Changes'}
      </button>
    </form>

    <!-- Tracks -->
    <div class="card bg-base-200 p-4 mt-6">
      <span class="text-sm font-semibold mb-3 block">Tracks ({playlist.tracks.length})</span>
      {#if playlist.tracks.length > 0}
        <div class="flex flex-col gap-1">
          {#each playlist.tracks as track, i (track.tid)}
            <div class="flex items-center gap-2 text-sm py-1">
              <span class="opacity-50 w-6 text-right">{i + 1}.</span>
              <a href="/track/{track.tid}" class="flex-1 hover:text-primary truncate">{track.title}</a>
              <span class="opacity-50 text-xs">{track.username}</span>
              <button class="btn btn-ghost btn-xs text-error" onclick={() => handleRemoveTrack(track.tid)}>×</button>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm opacity-60">No tracks in this playlist.</p>
      {/if}
    </div>

    {#if error}<p class="text-error text-sm mt-4">{error}</p>{/if}
    {#if message}<p class="text-success text-sm mt-4">{message}</p>{/if}

    <div class="mt-6 pt-4 border-t border-base-300">
      <button class="btn btn-sm btn-error btn-outline" onclick={handleDelete}>Delete Playlist</button>
    </div>
  {:else if error}
    <p class="text-error">{error}</p>
  {:else}
    <p class="opacity-60">Loading…</p>
  {/if}
</div>
