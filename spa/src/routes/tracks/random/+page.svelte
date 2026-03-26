<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Random Tracks — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-4">Random Tracks</h1>

{#if data.tracks.length > 0}
  <TrackList tracks={data.tracks} playbackContext={{ context: 'random' }} />
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No tracks found</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
