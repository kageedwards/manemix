<script lang="ts">
  import { auth, logout } from '$lib/stores/auth';
  import { translations, t, availableLocales, locale, setLocale } from '$lib/i18n';
  import ThemeToggle from './ThemeToggle.svelte';
  import SearchInput from './SearchInput.svelte';

  let menuOpen = $state(false);

  function toggleMenu() {
    menuOpen = !menuOpen;
  }

  async function handleLogout() {
    await logout();
  }
</script>

<nav class="navbar bg-base-200 border-b border-base-300 px-4 gap-2" aria-label="Main navigation">
  <!-- Site title -->
  <div class="flex-none">
    <a href="/" class="flex items-center gap-2 text-lg font-bold text-primary hover:opacity-80">
      <img src="/static/silhouette.png" alt="" class="nav-logo" width="24" height="23" aria-hidden="true" />
      {t($translations, 'site_name')}
    </a>
  </div>

  <!-- Desktop nav links -->
  <div class="hidden md:flex flex-1 items-center gap-4 ml-4">
    <a href="/" class="hover:text-primary">{t($translations, 'nav_home')}</a>
    <a href="/tracks/latest" class="hover:text-primary">{t($translations, 'nav_latest')}</a>
    <a href="/artists" class="hover:text-primary">{t($translations, 'nav_artists')}</a>
    <a href="/playlists" class="hover:text-primary">{t($translations, 'nav_playlists')}</a>
    <a href="/albums" class="hover:text-primary">{t($translations, 'nav_albums')}</a>
    <div class="ml-auto"><SearchInput /></div>
  </div>

  <!-- Desktop right section -->
  <div class="hidden md:flex items-center gap-2 ml-2">
    <ThemeToggle />
    {#if availableLocales.length > 1}
      <select
        class="select select-sm select-bordered"
        value={$locale}
        onchange={(e) => setLocale((e.target as HTMLSelectElement).value)}
        aria-label="Language"
      >
        {#each availableLocales as loc}
          <option value={loc}>{loc.toUpperCase()}</option>
        {/each}
      </select>
    {/if}
    {#if $auth.logged_in}
      <a href="/user/{$auth.uid}" class="text-sm hover:text-primary">{$auth.username}</a>
      <button class="btn btn-ghost btn-sm" onclick={handleLogout}>{t($translations, 'nav_logout')}</button>
    {:else}
      <a href="/login" class="btn btn-ghost btn-sm">{t($translations, 'nav_login')}</a>
    {/if}
  </div>

  <!-- Mobile hamburger -->
  <div class="flex md:hidden ml-auto items-center gap-2">
    <ThemeToggle />
    <button class="btn btn-ghost btn-sm" onclick={toggleMenu} aria-label="Toggle menu" aria-expanded={menuOpen}>
      <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        {#if menuOpen}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        {:else}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
        {/if}
      </svg>
    </button>
  </div>
</nav>

<!-- Mobile dropdown menu -->
{#if menuOpen}
  <div class="md:hidden bg-base-200 border-b border-base-300 px-4 py-3 flex flex-col gap-3">
    <a href="/" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_home')}</a>
    <a href="/tracks/latest" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_latest')}</a>
    <a href="/artists" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_artists')}</a>
    <a href="/playlists" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_playlists')}</a>
    <a href="/albums" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_albums')}</a>
    <SearchInput />
    {#if availableLocales.length > 1}
      <select
        class="select select-sm select-bordered"
        value={$locale}
        onchange={(e) => setLocale((e.target as HTMLSelectElement).value)}
        aria-label="Language"
      >
        {#each availableLocales as loc}
          <option value={loc}>{loc.toUpperCase()}</option>
        {/each}
      </select>
    {/if}
    {#if $auth.logged_in}
      <a href="/user/{$auth.uid}" class="hover:text-primary" onclick={() => menuOpen = false}>{$auth.username}</a>
      <button class="btn btn-ghost btn-sm text-left" onclick={handleLogout}>{t($translations, 'nav_logout')}</button>
    {:else}
      <a href="/login" class="hover:text-primary" onclick={() => menuOpen = false}>{t($translations, 'nav_login')}</a>
    {/if}
  </div>
{/if}
