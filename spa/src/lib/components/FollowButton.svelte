<script lang="ts">
  import { followUser, unfollowUser } from '$lib/api/client';

  interface Props {
    uid: number;
    isFollowing?: boolean;
  }
  let { uid, isFollowing = false }: Props = $props();

  let following = $state(isFollowing);
  let loading = $state(false);

  async function toggle() {
    if (loading) return;
    loading = true;
    const prev = following;
    following = !following; // optimistic
    try {
      if (following) {
        await followUser(uid);
      } else {
        await unfollowUser(uid);
      }
    } catch {
      following = prev; // revert
    } finally {
      loading = false;
    }
  }
</script>

<button
  class="btn btn-sm gap-1"
  class:btn-secondary={following}
  class:btn-ghost={!following}
  onclick={toggle}
  disabled={loading}
  aria-label={following ? 'Unfollow' : 'Follow'}
  aria-pressed={following}
>
  {following ? 'Following' : 'Follow'}
</button>
