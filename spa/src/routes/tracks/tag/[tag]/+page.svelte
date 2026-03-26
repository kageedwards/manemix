<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Tag: {data.tag} — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-4">Tag: <em>&ldquo;{data.tag}&rdquo;</em></h1>

{#if data.tracks.length > 0}
  <TrackList tracks={data.tracks} playbackContext={{ context: 'tag', param: data.tag }} />
{:else}
  <p class="opacity-60">No tracks tagged "{data.tag}".</p>
{/if}
