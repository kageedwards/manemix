<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth, setAuth } from '$lib/stores/auth';
  import { apiFetch, getMe } from '$lib/api/client';
  import { goto } from '$app/navigation';

  let name = $state('');
  let email = $state('');
  let about = $state('');
  let oldpw = $state('');
  let newpw = $state('');
  let newpwconf = $state('');
  let error = $state('');
  let message = $state('');
  let loading = $state(false);
  let loaded = $state(false);
  let showDeleteConfirm = $state(false);
  let deleteConfirmText = $state('');

  $effect(() => {
    if ($auth.logged_in && !loaded) {
      loadAccount();
    }
  });

  async function loadAccount() {
    try {
      const data = await apiFetch<{ username: string; email: string; about: string }>('/account');
      name = data.username;
      email = data.email;
      about = data.about;
      loaded = true;
    } catch {
      // Account endpoint may not return JSON yet — use auth store
      name = $auth.username ?? '';
      loaded = true;
    }
  }

  async function handleSubmit(e: Event) {
    e.preventDefault();
    error = '';
    message = '';
    if (newpw && newpw !== newpwconf) {
      error = 'Passwords do not match';
      return;
    }
    loading = true;
    try {
      await apiFetch('/account', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({
          name, email, about,
          ...(oldpw ? { oldpw, newpw, newpwconf } : {})
        }).toString()
      });
      const me = await getMe();
      setAuth(me);
      message = 'Changes saved';
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to save changes';
    } finally {
      loading = false;
    }
  }

  async function handleDelete() {
    try {
      await apiFetch('/account/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: new URLSearchParams({ confirm: 'Delete' }).toString()
      });
      setAuth({ logged_in: false });
      goto('/');
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to delete account';
    }
  }
</script>

<svelte:head>
  <title>Account — {$tt('site_name')}</title>
</svelte:head>

<div class="max-w-md mx-auto">
  {#if !$auth.logged_in}
    <p class="opacity-70">Please <a href="/login" class="text-primary hover:underline">log in</a> to manage your account.</p>
  {:else}
    <h1 class="text-xl font-bold mb-4">Account Settings</h1>

    <form onsubmit={handleSubmit} class="flex flex-col gap-4">
      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Display Name</span>
        <input type="text" bind:value={name} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Email</span>
        <input type="email" bind:value={email} class="input input-bordered w-full" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">About</span>
        <textarea bind:value={about} class="textarea textarea-bordered w-full" rows="4"></textarea>
      </div>

      <div class="divider text-xs opacity-50">Change Password (optional)</div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Current Password</span>
        <input type="password" bind:value={oldpw} class="input input-bordered w-full" autocomplete="current-password" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">New Password</span>
        <input type="password" bind:value={newpw} class="input input-bordered w-full" autocomplete="new-password" />
      </div>

      <div class="flex flex-col gap-1">
        <span class="text-sm font-medium">Confirm New Password</span>
        <input type="password" bind:value={newpwconf} class="input input-bordered w-full" autocomplete="new-password" />
      </div>

      {#if error}
        <p class="text-error text-sm">{error}</p>
      {/if}
      {#if message}
        <p class="text-success text-sm">{message}</p>
      {/if}

      <button type="submit" class="btn btn-primary" disabled={loading}>
        {loading ? 'Saving…' : 'Save Changes'}
      </button>
    </form>

    <div class="mt-8 pt-4 border-t border-base-300">
      {#if !showDeleteConfirm}
        <button class="btn btn-sm btn-error btn-outline" onclick={() => showDeleteConfirm = true}>Delete Account</button>
      {:else}
        <div class="card bg-error/10 border border-error/30 p-4 flex flex-col gap-3">
          <p class="text-sm font-semibold text-error">This will permanently delete your account and all your tracks.</p>
          <div class="flex flex-col gap-1">
            <span class="text-sm font-medium">Type "Delete" to confirm</span>
            <input type="text" bind:value={deleteConfirmText} class="input input-bordered input-sm w-full" placeholder="Delete" />
          </div>
          <div class="flex gap-2">
            <button class="btn btn-sm btn-error" disabled={deleteConfirmText !== 'Delete'} onclick={handleDelete}>Delete My Account</button>
            <button class="btn btn-sm btn-ghost" onclick={() => { showDeleteConfirm = false; deleteConfirmText = ''; }}>Cancel</button>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>
