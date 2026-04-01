<script lang="ts">
  import type { TickerItem } from '$lib/types/index.js';

  interface Props {
    items: TickerItem[];
  }
  let { items }: Props = $props();

  let currentIndex = $state(0);
  let dismissed = $state(false);

  $effect(() => {
    if (items.length <= 1) return;
    const interval = setInterval(() => {
      currentIndex = (currentIndex + 1) % items.length;
    }, 6000);
    return () => clearInterval(interval);
  });

  let current = $derived(items[currentIndex]);
</script>

{#if items.length > 0 && !dismissed}
  <div class="bg-primary/10 border border-primary/20 rounded-lg px-4 py-2 flex items-center gap-3 mb-6">
    <span class="text-primary text-sm font-medium shrink-0">News</span>
    <a href={current.url} class="text-sm hover:underline truncate flex-1">{current.title}</a>
    {#if items.length > 1}
      <span class="text-xs opacity-50 shrink-0">{currentIndex + 1}/{items.length}</span>
    {/if}
    <button
      class="btn btn-ghost btn-xs shrink-0"
      onclick={() => dismissed = true}
      aria-label="Dismiss news"
    >✕</button>
  </div>
{/if}
