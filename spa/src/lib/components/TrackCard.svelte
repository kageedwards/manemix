<script lang="ts">
  import type { Track, PlaybackContext } from '$lib/types/index.js';
  import { play, playerState, pause, resume } from '$lib/stores/player';
  import { translations, t } from '$lib/i18n';
  import { trackStatuses, monitorTrack, unmonitorTrack } from '$lib/stores/trackStatus';

  interface Props {
    track: Track;
    siblingTracks?: Track[];
    playbackContext?: PlaybackContext;
  }
  let { track, siblingTracks = [], playbackContext }: Props = $props();

  let ready = $derived($trackStatuses[track.tid]?.ready ?? true);
  let isCurrentTrack = $derived($playerState.currentTrack?.tid === track.tid);
  let isPlaying = $derived($playerState.currentTrack?.tid === track.tid && $playerState.isPlaying);

  $effect(() => {
    monitorTrack(track.tid);
    return () => unmonitorTrack(track.tid);
  });

  function handlePlay() {
    if (!ready) return;
    const queue = siblingTracks.length > 0 ? siblingTracks : [track];
    if (isPlaying) {
      pause();
    } else if (isCurrentTrack) {
      resume();
    } else {
      play(track, queue, playbackContext);
    }
  }
</script>

<div class="card card-side bg-base-200 p-3 gap-3 items-center">
  {#if track.has_art}
    <a href="/track/{track.tid}" class="shrink-0 relative w-14 h-14">
      <img
        src="/api/v1/track/{track.tid}/art/thumb"
        alt="{track.title} cover art"
        class="w-14 h-14 rounded object-cover"
        loading="lazy"
      />
      {#if isPlaying}
        <div class="absolute inset-0 rounded bg-black/50 flex items-end justify-center gap-[3px] pb-2">
          <span class="eq-bar eq-bar-1"></span>
          <span class="eq-bar eq-bar-2"></span>
          <span class="eq-bar eq-bar-3"></span>
          <span class="eq-bar eq-bar-4"></span>
        </div>
      {/if}
    </a>
  {:else}
    <a href="/track/{track.tid}" class="shrink-0 w-14 h-14 rounded bg-base-300 flex items-end justify-center gap-[3px] pb-2">
      {#if isPlaying}
        <span class="eq-bar eq-bar-1 playing"></span>
        <span class="eq-bar eq-bar-2 playing"></span>
        <span class="eq-bar eq-bar-3 playing"></span>
        <span class="eq-bar eq-bar-4 playing"></span>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="w-7 h-7 text-base-content opacity-30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" aria-hidden="true">
          <line x1="4" y1="18" x2="4" y2="10"/>
          <line x1="8" y1="18" x2="8" y2="4"/>
          <line x1="12" y1="18" x2="12" y2="8"/>
          <line x1="16" y1="18" x2="16" y2="6"/>
          <line x1="20" y1="18" x2="20" y2="12"/>
        </svg>
      {/if}
    </a>
  {/if}

  <div class="flex-1 min-w-0">
    <a href="/track/{track.tid}" class="font-semibold text-base-content hover:text-primary truncate block">
      {track.title}
    </a>
    <a href="/user/{track.uid}" class="text-sm opacity-70 hover:text-primary">
      {track.username}
    </a>
    <span class="text-xs opacity-50 ml-2">{track.date}</span>
    {#if !track.is_visible}
      <span class="badge badge-sm badge-outline border-warning text-warning bg-transparent ml-2">Unpublished</span>
    {/if}
    {#if !ready}
      <span class="text-xs text-warning ml-2">{t($translations, 'track_transcoding')}</span>
    {/if}
  </div>

  <button
    class="btn btn-circle btn-sm shrink-0"
    class:btn-primary={ready}
    class:cursor-not-allowed={!ready}
    onclick={handlePlay}
    disabled={!ready}
    style={!ready ? 'opacity: 0.7;' : ''}
    aria-label="{ready ? t($translations, 'player_play') : 'Transcoding'} {track.title}"
  >
    {#if isPlaying}
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6zM14 4h4v16h-4z"/></svg>
    {:else}
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
    {/if}
  </button>
</div>

<style>
  .eq-bar {
    display: inline-block;
    width: 3px;
    border-radius: 1px;
    background: currentColor;
    opacity: 0.8;
  }
  /* Static heights for overlay on art */
  .eq-bar-1 { animation: eq1 0.45s ease-in-out infinite alternate; }
  .eq-bar-2 { animation: eq2 0.55s ease-in-out infinite alternate; }
  .eq-bar-3 { animation: eq3 0.4s ease-in-out infinite alternate; }
  .eq-bar-4 { animation: eq4 0.5s ease-in-out infinite alternate; }

  /* Larger bars for the no-art placeholder */
  .eq-bar.playing {
    width: 4px;
    opacity: 0.4;
  }

  @keyframes eq1 { from { height: 6px; } to { height: 18px; } }
  @keyframes eq2 { from { height: 14px; } to { height: 8px; } }
  @keyframes eq3 { from { height: 8px; } to { height: 22px; } }
  @keyframes eq4 { from { height: 16px; } to { height: 10px; } }
</style>
