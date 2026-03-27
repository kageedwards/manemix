<script lang="ts">
  import '../app.css';
  import '../app.scss';
  import { page } from '$app/stores';
  import Navbar from '$lib/components/Navbar.svelte';
  import PlayerBar from '$lib/components/PlayerBar.svelte';
  import AudioVisualizer from '$lib/components/AudioVisualizer.svelte';
  import { visualizerEnabled } from '$lib/stores/visualizer';
  import { setAuth } from '$lib/stores/auth';
  import type { LayoutData } from './$types';

  let { data, children } = $props<{ data: LayoutData; children: any }>();
  let isDown = $derived($page.url.pathname === '/down');

  $effect(() => {
    if (data.auth) {
      setAuth(data.auth);
    }
  });
</script>

{#if isDown}
  {@render children()}
{:else}
  <Navbar />
  <main class="max-w-5xl mx-auto px-4 py-6">
    {@render children()}
  </main>
  {#if $visualizerEnabled}
    <AudioVisualizer />
  {/if}
  <PlayerBar />
{/if}
