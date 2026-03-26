<script lang="ts">
  import { tt } from '$lib/i18n';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { createAlbum } from '$lib/api/client';

  let name = $state('');
  let description = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!name.trim()) { error = 'Name is required'; return; }
    loading = true; error = '';
    try {
      const result = await createAlbum(name.trim(), description);
      goto(`/album/${result.id}/edit`);
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to create album';
    } finally { loading = false; }
  }
</script>

<svelte:head>
  <title>New Album — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-md mx-auto">
  {#if !$auth.logged_in}
    <p class="opacity-70">Please <a href="/login" class="text-primary hover:underline">log in</a> to create albums.</p>
  {:else}
    <h1 class="text-xl font-bold mb-4">Create Album</h1>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Name</span>
        <input type="text" bind:value={name} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Description (optional)</span>
        <textarea bind:value={description} class="textarea textarea-bordered w-full" rows="3"></textarea>
      </div>

      {#if error}<p class="text-error text-sm">{error}</p>{/if}

      <button type="submit" class="btn btn-primary" disabled={loading}>
        {loading ? 'Creating…' : 'Create Album'}
      </button>
    </form>
  {/if}
</div>
