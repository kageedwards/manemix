<script lang="ts">
  import { goto } from '$app/navigation';
  import { translations, t } from '$lib/i18n';

  let query = $state('');

  function handleSubmit(e: Event) {
    e.preventDefault();
    const q = query.trim();
    if (q) {
      goto(`/tracks/search?q=${encodeURIComponent(q)}`);
    }
  }
</script>

<form onsubmit={handleSubmit} class="flex items-center gap-1" role="search">
  <input
    type="text"
    bind:value={query}
    placeholder={t($translations, 'nav_search_placeholder')}
    class="input input-sm input-bordered w-40 md:w-56"
    aria-label={t($translations, 'nav_search_placeholder')}
  />
  <button type="submit" class="btn btn-ghost btn-sm" aria-label="Search">
    <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-4.35-4.35M11 19a8 8 0 100-16 8 8 0 000 16z" />
    </svg>
  </button>
</form>
