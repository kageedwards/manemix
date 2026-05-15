<script lang="ts">
  import type { EventItem } from '$lib/types/index.js';

  interface Props {
    events: EventItem[];
  }
  let { events }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  {#each events as event (event.event_id)}
    <div class="flex gap-2 items-start text-sm p-2 rounded bg-base-200">
      <span class="shrink-0 w-4 h-4 mt-0.5 text-base-content opacity-60" aria-hidden="true">
        {#if event.is_comment}
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
        {:else if event.is_favorite}
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
        {:else if event.is_follow}
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4"><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><line x1="19" y1="8" x2="19" y2="14"/><line x1="22" y1="11" x2="16" y2="11"/></svg>
        {:else if event.is_publish}
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        {:else}
          <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
        {/if}
      </span>
      <div class="flex-1 min-w-0">
        <span class="font-semibold">{event.source_name}</span>
        <span class="opacity-50 text-xs ml-1">{event.fuzzy_time}</span>
        {#if event.is_comment && event.message_html}
          <div class="mt-1 text-xs opacity-80">{@html event.message_html}</div>
        {/if}
        {#if event.has_track}
          <a href="/track/{event.tid}" class="text-xs text-primary hover:underline ml-1">{event.track_title}</a>
        {/if}
      </div>
    </div>
  {/each}
</div>
