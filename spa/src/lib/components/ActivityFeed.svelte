<script lang="ts">
  import type { EventItem } from '$lib/types/index.js';

  interface Props {
    events: EventItem[];
  }
  let { events }: Props = $props();

  function icon(e: EventItem): string {
    if (e.is_publish) return '🎵';
    if (e.is_comment) return '💬';
    if (e.is_favorite) return '⭐';
    return '👤';
  }

  function summary(e: EventItem): string {
    if (e.is_publish) return 'published';
    if (e.is_comment) return 'commented on';
    if (e.is_favorite) return 'favorited';
    return 'followed';
  }
</script>

<div class="flex flex-col">
  {#each events as event (event.event_id)}
    <div class="flex gap-2 items-start py-2 border-b border-base-200 last:border-0">
      <span class="shrink-0 text-sm" aria-hidden="true">{icon(event)}</span>
      <div class="flex-1 min-w-0 text-sm">
        <a href="/user/{event.source_uid}" class="font-medium hover:text-primary">{event.source_name}</a>
        <span class="opacity-60">{summary(event)}</span>
        {#if event.has_track}
          <a href="/track/{event.tid}" class="text-primary hover:underline">{event.track_title}</a>
        {:else if event.is_follow}
          <a href="/user/{event.target_uid}" class="text-primary hover:underline">{event.target_name}</a>
        {/if}
        <span class="text-xs opacity-40 ml-1">{event.fuzzy_time}</span>
      </div>
    </div>
  {/each}
  {#if events.length === 0}
    <p class="text-sm opacity-50 py-4">No recent activity yet.</p>
  {/if}
</div>
