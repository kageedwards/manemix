<script lang="ts">
  import { tt } from '$lib/i18n';
  import AlbumCard from '$lib/components/AlbumCard.svelte';
  import { auth } from '$lib/stores/auth';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Albums — {$tt('site_name')}</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
  <h1 class="text-xl font-bold">Albums</h1>
  {#if $auth.logged_in}
    <a href="/album/new" class="btn btn-sm btn-primary">Create Album</a>
  {/if}
</div>

{#if data.albums.length > 0}
  <div class="flex flex-col gap-2">
    {#each data.albums as album (album.playlist_id)}
      <AlbumCard {album} />
    {/each}
  </div>
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No published albums</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
