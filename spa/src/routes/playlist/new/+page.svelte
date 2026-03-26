<script lang="ts">
  import { tt } from '$lib/i18n';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { createPlaylist } from '$lib/api/client';

  let name = $state('');
  let description = $state('');
  let isPublic = $state(false);
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim()) { error = 'Name is required'; return; }
    loading = true; error = '';
    try {
      const result = await createPlaylist(name.trim(), description, isPublic);
      goto(`/playlist/${result.id}`);
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to create playlist';
    } finally { loading = false; }
  }
</script>

<svelte:head>
  <title>New Playlist — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-md mx-auto">
  {#if !$auth.logged_in}
    <p class="opacity-70">Please <a href="/login" class="text-primary hover:underline">log in</a> to create playlists.</p>
  {:else}
    <h1 class="text-xl font-bold mb-4">Create Playlist</h1>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Name</span>
        <input type="text" bind:value={name} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Description (optional)</span>
        <textarea bind:value={description} class="textarea textarea-bordered w-full" rows="3"></textarea>
      </div>

      <label class="flex items-center gap-2 cursor-pointer">
        <input type="checkbox" bind:checked={isPublic} class="checkbox checkbox-sm checkbox-primary" />
        <span class="text-sm">Make this playlist public</span>
      </label>

      {#if error}<p class="text-error text-sm">{error}</p>{/if}

      <button type="submit" class="btn btn-primary" disabled={loading}>
        {loading ? 'Creating…' : 'Create Playlist'}
      </button>
    </form>
  {/if}
</div>
