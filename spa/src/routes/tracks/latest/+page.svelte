<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Latest Tracks — {$tt('site_name')}</title>
  <link rel="alternate" type="application/atom+xml" title="Latest Tracks Feed" href="/tracks/latest/atom" />
</svelte:head>

<div class="flex items-center justify-between mb-4">
  <h1 class="text-xl font-bold">Latest Tracks</h1>
  <a href="/tracks/latest/atom" class="btn btn-ghost btn-xs opacity-60" title="Atom Feed">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6.18 15.64a2.18 2.18 0 010 4.36 2.18 2.18 0 010-4.36M4 4.44A15.56 15.56 0 0119.56 20h-2.83A12.73 12.73 0 004 7.27V4.44m0 5.66a9.9 9.9 0 019.9 9.9h-2.83A7.07 7.07 0 004 12.93v-2.83z"/></svg>
    Feed
  </a>
</div>

{#if data.tracks.length > 0}
  <TrackList tracks={data.tracks} playbackContext={{ context: 'latest' }} />
  <div class="flex gap-2 mt-4 justify-center">
    {#if data.page > 1}
      <a href="/tracks/latest?p={data.page - 1}" class="btn btn-sm btn-ghost">← Previous</a>
    {/if}
    <a href="/tracks/latest?p={data.page + 1}" class="btn btn-sm btn-ghost">Next →</a>
  </div>
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No tracks found</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
