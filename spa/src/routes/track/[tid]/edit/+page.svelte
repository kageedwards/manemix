<script lang="ts">
  import { tt } from '$lib/i18n';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { auth } from '$lib/stores/auth';
  import { apiFetch, getTrack } from '$lib/api/client';
  import TagInput from '$lib/components/TagInput.svelte';
  import LicensePicker from '$lib/components/LicensePicker.svelte';
  import type { ExtendedTrack } from '$lib/types/index.js';

  let tid = $derived(Number($page.params.tid));
  let isNew = $derived($page.url.searchParams.get('new') === '1');
  let track = $state<ExtendedTrack | null>(null);
  let title = $state('');
  let tags = $state('');
  let notes = $state('');
  let license = $state('Copyright');
  let customLicense = $state('');
  let mkdefault = $state(false);
  let retro = $state(false);
  let error = $state('');
  let message = $state('');
  let saving = $state(false);

  $effect(() => { loadTrack(); });

  async function loadTrack() {
    try {
      track = await getTrack(tid);
      title = track.title;
      tags = track.has_tags ? track.tags.join(', ') : '';
      notes = track.notes;
      const knownLicenses = ['Copyright', 'CC BY-NC', 'CC BY', 'CC BY-SA', 'CC BY-ND', 'CC BY-NC-SA', 'CC BY-NC-ND', 'Public Domain'];
      if (knownLicenses.includes(track.license)) { license = track.license; } else { license = 'custom'; customLicense = track.license; }
    } catch { error = 'Failed to load track'; }
  }

  async function handleSave(e: Event) {
    e.preventDefault();
    saving = true; error = ''; message = '';
    try {
      const finalLicense = license === 'custom' ? customLicense : license;
      await Promise.all([
        apiFetch(`/track/${tid}/rename`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ title })
        }),
        apiFetch(`/track/${tid}/tags`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ tags })
        }),
        apiFetch(`/track/${tid}/notes`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ notes })
        }),
        apiFetch(`/track/${tid}/license`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ license: finalLicense, mkdefault, retro })
        })
      ]);
      message = 'Saved';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save';
    } finally { saving = false; }
  }

  async function handleArtUpload(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files?.length) return;
    saving = true; error = ''; message = '';
    const formData = new FormData();
    formData.append('file', input.files[0]);
    try {
      await fetch(`/api/v1/track/${tid}/art/upload`, { method: 'POST', body: formData, credentials: 'include' });
      message = 'Art uploaded';
    } catch { error = 'Art upload failed'; }
    finally { saving = false; input.value = ''; }
  }

  async function handleArtDelete() {
    if (!confirm('Remove cover art?')) return;
    saving = true;
    try {
      await apiFetch(`/track/${tid}/art/delete`, { method: 'POST' });
      message = 'Art removed';
      if (track) track = { ...track, has_art: false };
    }
    catch { error = 'Failed to remove art'; }
    finally { saving = false; }
  }

  async function handleAudioReplace(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!input.files?.length) return;
    saving = true; error = ''; message = '';
    const formData = new FormData();
    formData.append('qqfile', input.files[0]);
    try {
      await fetch(`/api/v1/track/${tid}/upload`, { method: 'POST', body: formData, credentials: 'include' });
      message = 'Audio replaced — transcoding in progress';
    } catch { error = 'Audio upload failed'; }
    finally { saving = false; input.value = ''; }
  }

  async function handlePublish() {
    try {
      await apiFetch(`/track/${tid}/publish`, { method: 'POST' });
      goto(`/track/${tid}`);
    }
    catch (err: unknown) { error = err instanceof Error ? err.message : 'Failed to publish'; }
  }

  async function handleDelete() {
    if (!confirm('Delete this track permanently?')) return;
    try {
      await apiFetch(`/track/${tid}/delete`, { method: 'POST' });
      goto(`/user/${$auth.uid}`);
    } catch { error = 'Failed to delete'; }
  }
</script>

<svelte:head><title>Edit Track — {$tt('site_name')}</title></svelte:head>

<div class="max-w-lg mx-auto">
  <a href="/track/{tid}" class="text-sm text-primary hover:underline mb-4 block">← Back to track</a>

  {#if isNew}
    <div class="bg-info/10 border border-info/30 rounded-lg px-4 py-3 mb-6">
      <p class="text-sm font-medium">Your track was uploaded successfully.</p>
      <p class="text-sm opacity-70 mt-1">It's not published yet. Add a title, tags, notes, and cover art, then hit Publish when you're ready.</p>
    </div>
  {/if}

  {#if track}
    <h1 class="text-xl font-bold mb-6">Edit: {track.title}</h1>

    <form onsubmit={handleSave} class="flex flex-col gap-5">
      <!-- Metadata -->
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Title</span>
        <input type="text" bind:value={title} class="input input-bordered input-sm w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Tags</span>
        <TagInput bind:value={tags} />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Notes (BBCode)</span>
        <textarea bind:value={notes} class="textarea textarea-bordered text-sm w-full" rows="4"></textarea>
      </div>

      <!-- License -->
      <LicensePicker bind:value={license} bind:customValue={customLicense} />
      <div class="flex gap-4 mt-2">
        <label class="flex items-center gap-1 cursor-pointer text-xs">
          <input type="checkbox" bind:checked={mkdefault} class="checkbox checkbox-xs" /> Make default
        </label>
        <label class="flex items-center gap-1 cursor-pointer text-xs">
          <input type="checkbox" bind:checked={retro} class="checkbox checkbox-xs" /> Apply to all tracks
        </label>
      </div>

      <!-- Save button -->
      <button type="submit" class="btn btn-primary btn-sm" disabled={saving}>
        {saving ? 'Saving…' : 'Save All Changes'}
      </button>
    </form>

    <!-- Cover Art -->
    <div class="card bg-base-200 p-4 mt-6">
      <span class="text-sm font-semibold mb-2 block">Cover Art</span>
      <div class="flex items-start gap-4">
        {#if track.has_art}
          <img src="/api/v1/track/{tid}/art/thumb" alt="" class="w-20 h-20 rounded object-cover" />
        {/if}
        <div class="flex flex-col gap-2">
          <label class="btn btn-sm btn-ghost">
            {track.has_art ? 'Replace Art' : 'Upload Art'}
            <input type="file" accept="image/*" class="hidden" onchange={handleArtUpload} />
          </label>
          {#if track.has_art}
            <button class="btn btn-sm btn-ghost text-error" onclick={handleArtDelete}>Remove Art</button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Replace Audio -->
    <div class="card bg-base-200 p-4 mt-4">
      <span class="text-sm font-semibold mb-2 block">Replace Audio</span>
      <label class="btn btn-sm btn-ghost">
        Choose File
        <input type="file" accept="audio/*" class="hidden" onchange={handleAudioReplace} />
      </label>
    </div>

    {#if error}<p class="text-error text-sm mt-4">{error}</p>{/if}
    {#if message}<p class="text-success text-sm mt-4">{message}</p>{/if}

    <!-- Publish / Delete -->
    <div class="flex gap-2 mt-6 pt-4 border-t border-base-300">
      {#if !track.is_visible}
        <button class="btn btn-sm btn-primary" onclick={handlePublish}>Publish</button>
      {/if}
      <button class="btn btn-sm btn-error btn-outline" onclick={handleDelete}>Delete Track</button>
    </div>
  {:else if error}
    <p class="text-error">{error}</p>
  {:else}
    <p class="opacity-60">Loading…</p>
  {/if}
</div>
