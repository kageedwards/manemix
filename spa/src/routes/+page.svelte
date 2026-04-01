<script lang="ts">
  import { tt } from '$lib/i18n';
  import NewsBanner from '$lib/components/NewsBanner.svelte';
  import FeaturedCarousel from '$lib/components/FeaturedCarousel.svelte';
  import TrackList from '$lib/components/TrackList.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  let heroTracks = $derived(
    data.featured.length > 0 ? data.featured : data.latest.slice(0, 1)
  );
</script>

<svelte:head>
  <title>{$tt('site_name')}</title>
</svelte:head>

{#if data.ticker.length > 0}
  <NewsBanner items={data.ticker} />
{/if}

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
