<script lang="ts">
  import { postComment } from '$lib/api/client';
  import { auth } from '$lib/stores/auth';
  import { translations, t } from '$lib/i18n';

  interface Props {
    target: 'track' | 'user';
    id: number;
    onCommentPosted?: () => void;
  }
  let { target, id, onCommentPosted }: Props = $props();

  let message = $state('');
  let name = $state('');
  let honeypot = $state('');
  let error = $state('');
  let loading = $state(false);

  async function handleSubmit(e: Event) {
    e.preventDefault();
    if (!message.trim() || honeypot) return;

    loading = true;
    error = '';
    try {
      const nameVal = $auth.logged_in ? undefined : (name.trim() || undefined);
      await postComment(target, id, message.trim(), nameVal);
      message = '';
      name = '';
      onCommentPosted?.();
    } catch (err: unknown) {
      error = err instanceof Error ? err.message : 'Failed to post comment';
    } finally {
      loading = false;
    }
  }
</script>

<form onsubmit={handleSubmit} class="flex flex-col gap-2">
  {#if !$auth.logged_in}
    <input
      type="text"
      bind:value={name}
      placeholder="Name (optional)"
      class="input input-sm input-bordered"
    />
  {/if}

  <!-- Honeypot field -->
  <input type="text" bind:value={honeypot} class="hidden" tabindex="-1" autocomplete="off" aria-hidden="true" />

  <textarea
    bind:value={message}
    placeholder={t($translations, 'track_post_comment')}
    class="textarea textarea-bordered text-sm"
    rows="3"
    required
  ></textarea>

  {#if error}
    <p class="text-error text-xs">{error}</p>
  {/if}

  <button type="submit" class="btn btn-sm btn-primary self-end" disabled={loading || !message.trim()}>
    {loading ? 'Posting…' : t($translations, 'track_post_comment')}
  </button>
</form>
