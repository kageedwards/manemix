<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Search: {data.query} — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-4">Search: <em>&ldquo;{data.query}&rdquo;</em></h1>

{#if data.tracks.length > 0}
  <TrackList tracks={data.tracks} playbackContext={{ context: 'search', param: data.query }} />
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No results found</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
