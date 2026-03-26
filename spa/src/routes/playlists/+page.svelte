<script lang="ts">
  import { tt } from '$lib/i18n';
  import PlaylistCard from '$lib/components/PlaylistCard.svelte';
  import { auth } from '$lib/stores/auth';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>Playlists — {$tt('site_name')}</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
  <h1 class="text-xl font-bold">Playlists</h1>
  {#if $auth.logged_in}
    <a href="/playlist/new" class="btn btn-sm btn-primary">Create Playlist</a>
  {/if}
</div>

{#if data.playlists.length > 0}
  <div class="flex flex-col gap-2">
    {#each data.playlists as playlist (playlist.playlist_id)}
      <PlaylistCard {playlist} />
    {/each}
  </div>
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No public playlists</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
