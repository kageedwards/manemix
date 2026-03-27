<script lang="ts">
  import { playerState, pause, resume, next, prev, seek, setVolume } from '$lib/stores/player';
  import { translations, t } from '$lib/i18n';
  import VisualizerToggle from './VisualizerToggle.svelte';

  const interactiveTags = new Set(['INPUT', 'TEXTAREA', 'SELECT']);

  function handleKeydown(e: KeyboardEvent) {
    if (!$playerState.currentTrack) return;
    const tag = (e.target as HTMLElement)?.tagName;
    if (interactiveTags.has(tag)) return;
    if ((e.target as HTMLElement)?.isContentEditable) return;

    if (e.key === ' ') {
      e.preventDefault();
      e.stopPropagation();
      togglePlayPause();
    }
  }

  $effect(() => {
    if (typeof window === 'undefined') return;
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  function formatTime(seconds: number): string {
    if (!isFinite(seconds) || seconds < 0) return '0:00';
    const m = Math.floor(seconds / 60);
    const s = Math.floor(seconds % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  function handleSeek(e: Event) {
    const input = e.target as HTMLInputElement;
    seek(parseFloat(input.value));
  }

  function handleVolume(e: Event) {
    const input = e.target as HTMLInputElement;
    setVolume(parseFloat(input.value));
  }

  function togglePlayPause() {
    if ($playerState.isPlaying) {
      pause();
    } else {
      resume();
    }
  }

  let progress = $derived(
    $playerState.duration > 0 ? ($playerState.currentTime / $playerState.duration) * 100 : 0
  );
</script>

{#if $playerState.currentTrack}
  <!-- Desktop player -->
  <div class="fixed bottom-0 left-0 right-0 bg-base-300 border-t border-base-content/10 z-50 hidden md:flex items-center px-4 gap-4" style="height: var(--player-height)">
    <!-- Track info -->
    <div class="flex items-center gap-3 min-w-0 w-56 shrink-0">
      {#if $playerState.currentTrack.has_art}
        <img
          src="/api/v1/track/{$playerState.currentTrack.tid}/art/thumb"
          alt=""
          class="w-10 h-10 rounded object-cover shrink-0"
        />
      {/if}
      <div class="min-w-0">
        <a href="/track/{$playerState.currentTrack.tid}" class="text-sm font-semibold truncate block hover:text-primary">
          {$playerState.currentTrack.title}
        </a>
        <a href="/user/{$playerState.currentTrack.uid}" class="text-xs opacity-60 truncate block hover:text-primary">
          {$playerState.currentTrack.username}
        </a>
      </div>
    </div>

    <!-- Controls -->
    <div class="flex items-center gap-2">
      <button class="btn btn-ghost btn-sm btn-circle" onclick={prev} aria-label={t($translations, 'player_prev')}>
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
      </button>
      <button class="btn btn-primary btn-sm btn-circle" onclick={togglePlayPause} aria-label={$playerState.isPlaying ? t($translations, 'player_pause') : t($translations, 'player_play')}>
        {#if $playerState.isPlaying}
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6zM14 4h4v16h-4z"/></svg>
        {:else}
          <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
        {/if}
      </button>
      <button class="btn btn-ghost btn-sm btn-circle" onclick={next} aria-label={t($translations, 'player_next')}>
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
      </button>
    </div>

    <!-- Seek bar -->
    <div class="flex items-center gap-2 flex-1">
      <span class="text-xs opacity-60 w-10 text-right tabular-nums">{formatTime($playerState.currentTime)}</span>
      <input
        type="range"
        min="0"
        max={$playerState.duration || 0}
        step="0.1"
        value={$playerState.currentTime}
        oninput={handleSeek}
        class="range range-xs range-primary flex-1"
        aria-label="Seek"
      />
      <span class="text-xs opacity-60 w-10 tabular-nums">{formatTime($playerState.duration)}</span>
    </div>

    <!-- Volume -->
    <div class="flex items-center gap-1 w-28 shrink-0">
      <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 opacity-60" viewBox="0 0 24 24" fill="currentColor">
        <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3A4.5 4.5 0 0014 8.5v7a4.47 4.47 0 002.5-3.5z"/>
      </svg>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={$playerState.volume}
        oninput={handleVolume}
        class="range range-xs flex-1"
        aria-label="Volume"
      />
    </div>

    <!-- Visualizer toggle -->
    <VisualizerToggle />
  </div>

  <!-- Mobile compact player -->
  <div class="fixed bottom-0 left-0 right-0 bg-base-300 border-t border-base-content/10 z-50 flex md:hidden items-center px-3 gap-2" style="height: var(--player-height-compact)">
    <div class="flex-1 min-w-0">
      <span class="text-sm font-semibold truncate block">{$playerState.currentTrack.title}</span>
    </div>
    <!-- Mini progress bar -->
    <div class="absolute top-0 left-0 right-0 h-0.5 bg-base-content/10">
      <div class="h-full bg-primary transition-all" style="width: {progress}%"></div>
    </div>
    <button class="btn btn-primary btn-sm btn-circle shrink-0" onclick={togglePlayPause} aria-label={$playerState.isPlaying ? t($translations, 'player_pause') : t($translations, 'player_play')}>
      {#if $playerState.isPlaying}
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M6 4h4v16H6zM14 4h4v16h-4z"/></svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
      {/if}
    </button>
    <VisualizerToggle />
  </div>
{/if}
