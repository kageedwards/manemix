<script lang="ts">
  import type { EventItem } from '$lib/types/index.js';

  interface Props {
    events: EventItem[];
  }
  let { events }: Props = $props();

  function eventIcon(event: EventItem): string {
    if (event.is_comment) return '💬';
    if (event.is_favorite) return '⭐';
    if (event.is_follow) return '👤';
    if (event.is_publish) return '🎵';
    return '📌';
  }
</script>

<div class="flex flex-col gap-2">
  {#each events as event (event.event_id)}
    <div class="flex gap-2 items-start text-sm p-2 rounded bg-base-200">
      <span class="shrink-0" aria-hidden="true">{eventIcon(event)}</span>
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
