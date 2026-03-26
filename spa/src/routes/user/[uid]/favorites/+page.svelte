<script lang="ts">
  import { tt } from '$lib/i18n';
  import TrackList from '$lib/components/TrackList.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Favorites — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-4">Favorites</h1>
<a href="/user/{data.uid}" class="text-sm text-primary hover:underline mb-4 block">← Back to profile</a>

{#if data.tracks.length > 0}
  <TrackList tracks={data.tracks} playbackContext={{ context: 'favorites', param: String(data.uid) }} />
{:else}
  <p class="opacity-60">No favorites yet.</p>
{/if}
