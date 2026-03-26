<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>{$tt('site_name')}</title>
</svelte:head>

{#if data.featured.length > 0}
  <section class="mb-8">
    <h2 class="text-xl font-bold mb-3">Featured</h2>
    <TrackList tracks={data.featured} playbackContext={{ context: 'featured' }} />
  </section>
{/if}

{#if data.latest.length > 0}
  <section class="mb-8">
    <h2 class="text-xl font-bold mb-3">Latest</h2>
    <TrackList tracks={data.latest} playbackContext={{ context: 'latest' }} />
  </section>
{/if}

{#if data.random.length > 0}
  <section class="mb-8">
    <h2 class="text-xl font-bold mb-3">Random</h2>
    <TrackList tracks={data.random} playbackContext={{ context: 'random' }} />
  </section>
{/if}
