<script lang="ts">
  import type { Track, PlaybackContext } from '$lib/types/index.js';
  import TrackCard from './TrackCard.svelte';

  interface Props {
    tracks: Track[];
    playbackContext?: PlaybackContext;
  }
  let { tracks, playbackContext }: Props = $props();
</script>

<div class="flex flex-col gap-2">
  {#each tracks as track (track.tid)}
    <TrackCard {track} siblingTracks={tracks} {playbackContext} />
  {/each}
  {#if tracks.length === 0}
    <p class="text-sm opacity-50 py-4">No tracks to show.</p>
  {/if}
</div>
