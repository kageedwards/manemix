<script lang="ts">
  import { tt } from '$lib/i18n';
  import { page } from '$app/stores';
</script>

<svelte:head>
  <title>Error {$page.status} — {$tt('site_name')}</title>
</svelte:head>

<div class="flex flex-col items-center justify-center py-20 gap-4">
  <h1 class="text-4xl font-bold text-primary">{$page.status}</h1>
  <p class="text-lg opacity-70">{$page.error?.message ?? 'Something went wrong'}</p>
  <a href="/" class="btn btn-sm btn-ghost mt-4">← Back to Home</a>
</div>
