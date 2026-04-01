<script lang="ts">
  import { tt } from '$lib/i18n';
  import NewsBanner from '$lib/components/NewsBanner.svelte';
  import FeaturedCarousel from '$lib/components/FeaturedCarousel.svelte';
  import TrackList from '$lib/components/TrackList.svelte';
  import ActivityFeed from '$lib/components/ActivityFeed.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
</script>

<svelte:head>
  <title>{$tt('site_name')}</title>
</svelte:head>

{#if data.ticker.length > 0}
  <NewsBanner items={data.ticker} />
{/if}

<FeaturedCarousel tracks={data.featured} />

<div class="grid grid-cols-1 lg:grid-cols-5 gap-8">
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
