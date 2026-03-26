<script lang="ts">
  import { tt } from '$lib/i18n';
  import ArtistCard from '$lib/components/ArtistCard.svelte';
  import { searchArtists } from '$lib/api/client';
  import type { Artist } from '$lib/types/index.js';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
  let query = $state('');
  let results = $state<Artist[] | null>(null);
  let searching = $state(false);

  async function handleSearch(e: Event) {
    e.preventDefault();
    const q = query.trim();
    if (!q) { results = null; return; }
    searching = true;
    try {
      results = await searchArtists(q);
    } catch {
      results = [];
    } finally {
      searching = false;
    }
  }

  let displayArtists = $derived(results ?? data.artists);
</script>

<svelte:head>
  <title>Artists — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-4">Artists</h1>

<form onsubmit={handleSearch} class="flex gap-2 mb-4">
  <input type="text" bind:value={query} placeholder="Search artists…" class="input input-sm input-bordered flex-1" />
  <button type="submit" class="btn btn-sm btn-primary" disabled={searching}>Search</button>
</form>

{#if results !== null && results.length === 0}
  <p class="opacity-60 mb-4">No artists found for "{query}".</p>
{/if}

<div class="flex flex-col gap-2">
  {#each displayArtists as artist (artist.uid)}
    <ArtistCard {artist} />
  {/each}
</div>
