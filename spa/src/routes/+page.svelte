<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth } from '$lib/stores/auth';
  import NewsBanner from '$lib/components/NewsBanner.svelte';
  import FeaturedCarousel from '$lib/components/FeaturedCarousel.svelte';
  import TrackList from '$lib/components/TrackList.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let heroTracks = $derived(
    data.featured.length > 0 ? data.featured : data.latest.slice(0, 1)
  );

  let maxCount = $derived(Math.max(...data.tags.map(t => t.count), 1));
  let minCount = $derived(Math.min(...data.tags.map(t => t.count), 1));

  function fontSize(count: number): string {
    if (maxCount === minCount) return '1rem';
    const ratio = (count - minCount) / (maxCount - minCount);
    return `${0.75 + ratio * 2}rem`;
  }
</script>

<svelte:head>
  <title>{$tt('site_name')}</title>
</svelte:head>

{#if data.ticker.length > 0}
  <NewsBanner items={data.ticker} />
{/if}

<div class="min-h-[100vh]">
  <FeaturedCarousel tracks={heroTracks} />

  <div class="content-panel relative rounded-t-xl pt-6 px-4 lg:px-6">
    <div class="grid grid-cols-1 lg:grid-cols-5 gap-8 relative z-10">
      <!-- Latest tracks (wider column) -->
      <section class="lg:col-span-3">
        <div class="flex items-center justify-between mb-3">
          <h2 class="text-xl font-bold">Latest</h2>
          <a href="/tracks/latest" class="text-sm text-primary hover:underline">View all</a>
        </div>
        <TrackList tracks={data.latest} playbackContext={{ context: 'latest' }} />
      </section>

      <!-- Community activity (narrower column) -->
      <section class="lg:col-span-2">
        <h2 class="text-xl font-bold mb-3">Community</h2>
        <ActivityFeed events={data.events} />
      </section>
    </div>
  </div>
</div>

<!-- Below the fold: intro for guests, tag cloud for logged-in users -->
{#if $auth.logged_in}
  {#if data.tags.length > 0}
    <section class="max-w-3xl mx-auto px-4 py-12 text-center">
      <h2 class="text-xl font-bold mb-6">Explore Tags</h2>
      <div class="flex flex-wrap gap-3 items-baseline justify-center">
        {#each data.tags as item (item.tag)}
          <a
            href="/tracks/tag/{item.tag}"
            class="hover:text-primary transition-colors"
            style="font-size: {fontSize(item.count)}; opacity: {0.5 + (item.count - minCount) / (maxCount - minCount) * 0.5}"
          >
            {item.tag}
          </a>
        {/each}
      </div>
    </section>
  {/if}
{:else}
  <section class="max-w-2xl mx-auto px-4 py-12 text-center">
    <h2 class="text-2xl font-bold mb-4">Share your music with the world</h2>
    <p class="text-base opacity-70 mb-3">
      Favorite tracks, follow artists, and create playlists. Upload your own music for everyone to listen to.
    </p>
    <p class="text-base opacity-70 mb-6">
      No limits on formats, filesize, or downloads. Tracks play in the best conditions on every device and browser.
    </p>
    <div class="flex gap-3 justify-center">
      <a href="/register" class="btn btn-primary">Get Started</a>
      <a href="/login" class="btn btn-ghost">Login</a>
    </div>
  </section>
{/if}

<style>
  .content-panel::before {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--color-base-300);
    opacity: 0.6;
    mask-image: linear-gradient(to bottom, black 0%, transparent 50%);
    -webkit-mask-image: linear-gradient(to bottom, black 0%, transparent 50%);
    pointer-events: none;
  }
</style>
