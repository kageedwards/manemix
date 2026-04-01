<script lang="ts">
  import { tt } from '$lib/i18n';
  import { goto } from '$app/navigation';
  import { setAuth } from '$lib/stores/auth';
  import { getMe, apiFetch } from '$lib/api/client';
  import { translations, t } from '$lib/i18n';

  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    loading = true;
    try {
      await apiFetch('/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ email, pw: password }).toString()
      });
      const me = await getMe();
      setAuth(me);
      goto('/');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Login failed';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Login — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-sm mx-auto">
  <h1 class="text-xl font-bold mb-4">{t($translations, 'nav_login')}</h1>

  <form onsubmit={handleSubmit} class="flex flex-col gap-3">
    <label class="form-control">
      <span class="label-text text-sm mb-1">Email</span>
      <input type="email" bind:value={email} class="input input-bordered" required autocomplete="email" />
    </label>

    <label class="form-control">
      <span class="label-text text-sm mb-1">Password</span>
      <input type="password" bind:value={password} class="input input-bordered" required autocomplete="current-password" />
    </label>

    {#if error}
      <p class="text-error text-sm">{error}</p>
    {/if}

    <button type="submit" class="btn btn-primary" disabled={loading}>
      {loading ? 'Logging in…' : t($translations, 'nav_login')}
    </button>

    <p class="text-sm opacity-70">
      Don't have an account? <a href="/register" class="text-primary hover:underline">Register</a>
    </p>
  </form>
</div>
