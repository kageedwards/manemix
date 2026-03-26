<script lang="ts">
  import { favoriteTrack, unfavoriteTrack } from '$lib/api/client';

  interface Props {
    tid: number;
    isFavorited?: boolean;
  }
  let { tid, isFavorited = false }: Props = $props();

  let favorited = $state(isFavorited);
  let loading = $state(false);

  async function toggle() {
    if (loading) return;
    loading = true;
    const prev = favorited;
    favorited = !favorited; // optimistic
    try {
      if (favorited) {
        await favoriteTrack(tid);
      } else {
        await unfavoriteTrack(tid);
      }
    } catch {
      favorited = prev; // revert
    } finally {
      loading = false;
    }
  }
</script>

<button
  class="btn btn-sm gap-1"
  class:btn-primary={favorited}
  class:btn-ghost={!favorited}
  onclick={toggle}
  disabled={loading}
  aria-label={favorited ? 'Unfavorite' : 'Favorite'}
  aria-pressed={favorited}
>
  <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 24 24" fill={favorited ? 'currentColor' : 'none'} stroke="currentColor" stroke-width="2">
    <path d="M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 22 12 18.56 5.82 22 7 14.14l-5-4.87 6.91-1.01L12 2z"/>
  </svg>
  {favorited ? 'Favorited' : 'Favorite'}
</button>
