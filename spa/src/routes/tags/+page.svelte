<script lang="ts">
  import { tt } from '$lib/i18n';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  // Calculate font sizes relative to usage count
  let maxCount = $derived(Math.max(...data.tags.map(t => t.count), 1));
  let minCount = $derived(Math.min(...data.tags.map(t => t.count), 1));

  function fontSize(count: number): string {
    if (maxCount === minCount) return '1rem';
    const ratio = (count - minCount) / (maxCount - minCount);
    const size = 0.75 + ratio * 2; // 0.75rem to 2.75rem
    return `${size}rem`;
  }
</script>

<svelte:head>
  <title>Tags — {$tt('site_name')}</title>
</svelte:head>

<h1 class="text-xl font-bold mb-6">Tags</h1>

{#if data.tags.length > 0}
  <div class="flex flex-wrap gap-3 items-baseline justify-center py-8">
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
{:else}
  <div class="flex flex-col items-center justify-center py-20 gap-4">
    <h1 class="text-4xl font-bold text-primary">No tags yet</h1>
    <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
  </div>
{/if}
