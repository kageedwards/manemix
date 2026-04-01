<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth } from '$lib/stores/auth';
  import { goto } from '$app/navigation';

  let files = $state<FileList | null>(null);
  let error = $state('');
  let uploading = $state(false);
  let progress = $state('');

  async function handleUpload(e: Event) {
    e.preventDefault();
    if (!files?.length) return;
    uploading = true;
    error = '';
    progress = '';

    for (let i = 0; i < files.length; i++) {
      const file = files[i];
      progress = files.length > 1 ? `Uploading ${i + 1} of ${files.length}: ${file.name}` : `Uploading: ${file.name}`;

      const formData = new FormData();
      formData.append('qqfile', file);

      try {
        const res = await fetch('/api/v1/track/new', {
          method: 'POST',
          body: formData,
          credentials: 'include',
          headers: { 'X-Requested-With': 'XMLHttpRequest' }
        });
        const data = await res.json();
        if (data.success && files.length === 1) {
          goto(`/track/${data.tid}/edit?new=1`);
          return;
        } else if (!data.success) {
          error = `Failed to upload ${file.name}`;
        }
      } catch {
        error = `Failed to upload ${file.name}`;
      }
    }

    uploading = false;
    if (!error && files.length > 1) {
      goto(`/user/${$auth.uid}`);
    }
  }
</script>

<svelte:head>
  <title>Upload — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-md mx-auto">
  {#if !$auth.logged_in}
    <p class="opacity-70">Please <a href="/login" class="text-primary hover:underline">log in</a> to upload tracks.</p>
  {:else}
    <h1 class="text-xl font-bold mb-4">Upload Track</h1>

    <form onsubmit={handleUpload} class="flex flex-col gap-4">
      <label class="form-control">
        <span class="label-text text-sm mb-1">Audio File(s)</span>
        <input type="file" accept="audio/*" multiple bind:files class="file-input file-input-bordered w-full" />
      </label>

      <p class="text-xs opacity-50">Supported formats: MP3, FLAC, OGG, WAV, AAC, and more. Multiple files can be selected for batch upload.</p>

      {#if progress}
        <p class="text-sm opacity-70">{progress}</p>
      {/if}
      {#if error}
        <p class="text-error text-sm">{error}</p>
      {/if}

      <button type="submit" class="btn btn-primary" disabled={uploading || !files?.length}>
        {uploading ? 'Uploading…' : 'Upload'}
      </button>
    </form>
  {/if}
</div>
