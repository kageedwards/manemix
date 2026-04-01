<script lang="ts">
  import type { Track, PlaybackContext } from '$lib/types/index.js';
  import { play, playerState, pause, resume } from '$lib/stores/player';
  import { trackStatuses, monitorTrack, unmonitorTrack } from '$lib/stores/trackStatus';

  interface Props {
    tracks: Track[];
  }
  let { tracks }: Props = $props();

  let currentIndex = $state(0);
  let track = $derived(tracks[currentIndex]);
  let ready = $derived($trackStatuses[track?.tid]?.ready ?? true);
  let isCurrentTrack = $derived($playerState.currentTrack?.tid === track?.tid);
  let isPlaying = $derived(isCurrentTrack && $playerState.isPlaying);

  const playbackContext: PlaybackContext = { context: 'featured' };

  $effect(() => {
    tracks.forEach(t => monitorTrack(t.tid));
    return () => tracks.forEach(t => unmonitorTrack(t.tid));
  });

  function prev() {
    currentIndex = (currentIndex - 1 + tracks.length) % tracks.length;
  }
  function next() {
    currentIndex = (currentIndex + 1) % tracks.length;
  }
  function handlePlay() {
    if (!ready) return;
    if (isPlaying) { pause(); }
    else if (isCurrentTrack) { resume(); }
    else { play(track, tracks, playbackContext); }
  }
</script>

{#if tracks.length > 0}
<section class="hero-carousel relative w-full overflow-hidden mb-8">
  <!-- Background art (blurred) -->
  <div class="absolute inset-0 z-0">
    {#if track.has_art}
      <img
        src="/api/v1/track/{track.tid}/art"
        alt=""
        class="w-full h-full object-cover blur-2xl scale-110 opacity-40"
        aria-hidden="true"
      />
      <div class="absolute inset-0 bg-gradient-to-t from-base-100 via-base-100/60 to-transparent"></div>
    {/if}
  </div>

  <!-- Content -->
  <div class="relative z-10 flex flex-col md:flex-row items-center gap-6 py-10 md:py-14">
    <!-- Art -->
    <a href="/track/{track.tid}" class="shrink-0">
      {#if track.has_art}
        <img
          src="/api/v1/track/{track.tid}/art/medium"
          alt="{track.title} cover art"
          class="w-48 h-48 md:w-56 md:h-56 rounded-lg object-cover shadow-lg"
        />
      {:else}
        <div class="w-48 h-48 md:w-56 md:h-56 rounded-lg bg-base-300 flex items-center justify-center">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16 opacity-20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
            <line x1="4" y1="18" x2="4" y2="10"/><line x1="8" y1="18" x2="8" y2="4"/>
            <line x1="12" y1="18" x2="12" y2="8"/><line x1="16" y1="18" x2="16" y2="6"/>
            <line x1="20" y1="18" x2="20" y2="12"/>
          </svg>
        </div>
      {/if}
    </a>

    <!-- Info -->
    <div class="flex-1 min-w-0 text-center md:text-left">
      <p class="text-xs uppercase tracking-wider opacity-50 mb-1">Featured</p>
      <a href="/track/{track.tid}" class="text-2xl md:text-3xl font-bold hover:text-primary block truncate">
        {track.title}
      </a>
      <a href="/user/{track.uid}" class="text-lg opacity-70 hover:text-primary">
        {track.username}
      </a>
      <div class="mt-4 flex items-center gap-3 justify-center md:justify-start">
        <button
          class="btn btn-primary btn-lg gap-2"
          class:cursor-not-allowed={!ready}
          onclick={handlePlay}
          disabled={!ready}
          aria-label="{isPlaying ? 'Pause' : 'Play'} {track.title}"
        >
          {#if isPlaying}
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6zM14 4h4v16h-4z"/></svg>
            Pause
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            Play
          {/if}
        </button>
      </div>
    </div>

    <!-- Nav arrows -->
    {#if tracks.length > 1}
      <div class="flex md:flex-col gap-2 shrink-0">
        <button class="btn btn-circle btn-sm btn-ghost" onclick={prev} aria-label="Previous featured track">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 18l-6-6 6-6"/></svg>
        </button>
        <button class="btn btn-circle btn-sm btn-ghost" onclick={next} aria-label="Next featured track">
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18l6-6-6-6"/></svg>
        </button>
      </div>
    {/if}
  </div>

  <!-- Dots -->
  {#if tracks.length > 1}
    <div class="relative z-10 flex justify-center gap-1.5 pb-4">
      {#each tracks as _, i}
        <button
          class="w-2 h-2 rounded-full transition-colors"
          class:bg-primary={i === currentIndex}
          class:bg-base-content={i !== currentIndex}
          class:opacity-30={i !== currentIndex}
          onclick={() => currentIndex = i}
          aria-label="Go to featured track {i + 1}"
        ></button>
      {/each}
    </div>
  {/if}
</section>
{/if}

<style>
  .hero-carousel {
    min-height: 30vh;
    max-height: 45vh;
  }
</style>
