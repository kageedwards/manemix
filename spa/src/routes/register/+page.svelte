<script lang="ts">
  import { tt } from '$lib/i18n';
  import { goto } from '$app/navigation';
  import { setAuth } from '$lib/stores/auth';
  import { getMe, apiFetch } from '$lib/api/client';

  let name = $state('');
  let email = $state('');
  let password = $state('');
  let confirm = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    if (password !== confirm) {
      error = 'Passwords do not match';
      return;
    }
    loading = true;
    try {
      await apiFetch('/register', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ name, email, pw: password, pwconf: confirm }).toString()
      });
      const me = await getMe();
      setAuth(me);
      goto('/');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Registration failed';
    } finally {
      loading = false;
    }
  }
</script>

<svelte:head>
  <title>Register — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-sm mx-auto">
  <h1 class="text-xl font-bold mb-4">Register</h1>

  <form onsubmit={handleSubmit} class="flex flex-col gap-3">
    <label class="form-control">
      <span class="label-text text-sm mb-1">Display Name</span>
      <input type="text" bind:value={name} class="input input-bordered" required autocomplete="username" />
    </label>

    <label class="form-control">
      <span class="label-text text-sm mb-1">Email</span>
      <input type="email" bind:value={email} class="input input-bordered" required autocomplete="email" />
    </label>

    <label class="form-control">
      <span class="label-text text-sm mb-1">Password</span>
      <input type="password" bind:value={password} class="input input-bordered" required autocomplete="new-password" />
    </label>

    <label class="form-control">
      <span class="label-text text-sm mb-1">Confirm Password</span>
      <input type="password" bind:value={confirm} class="input input-bordered" required autocomplete="new-password" />
    </label>

    {#if error}
      <p class="text-error text-sm">{error}</p>
    {/if}

    <button type="submit" class="btn btn-primary" disabled={loading}>
      {loading ? 'Registering…' : 'Register'}
    </button>

    <p class="text-sm opacity-70">
      Already have an account? <a href="/login" class="text-primary hover:underline">Login</a>
    </p>
  </form>
</div>
