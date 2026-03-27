<script lang="ts">
  import { visualizerEnabled, toggleVisualizer } from '$lib/stores/visualizer';
  import { translations, t } from '$lib/i18n';
</script>

<button
  class="btn btn-ghost btn-sm"
  onclick={toggleVisualizer}
  aria-label={t($translations, 'visualizer_toggle')}
  title={t($translations, 'visualizer_toggle')}
>
  <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
    <rect x="3" y="14" width="3" height="6" rx="1" />
    <rect x="8" y="8" width="3" height="12" rx="1" />
    <rect x="13" y="4" width="3" height="16" rx="1" />
    <rect x="18" y="10" width="3" height="10" rx="1" />
    {#if !$visualizerEnabled}
      <line x1="2" y1="2" x2="22" y2="22" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" />
    {/if}
  </svg>
</button>
