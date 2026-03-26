<script lang="ts">
  import { theme, toggleTheme } from '$lib/stores/theme';
  import { translations, t } from '$lib/i18n';
</script>

<button
  class="btn btn-ghost btn-sm"
  onclick={toggleTheme}
  aria-label={t($translations, 'theme_toggle')}
  title={t($translations, 'theme_toggle')}
>
  {#if $theme === 'dark'}
    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
      <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
    </svg>
  {:else}
    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12.79A9 9 0 1111.21 3a7 7 0 009.79 9.79z" />
    </svg>
  {/if}
</button>
