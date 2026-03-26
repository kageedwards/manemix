<script lang="ts">
  import type { Artist } from '$lib/types/index.js';

  interface Props {
    artist: Artist;
  }
  let { artist }: Props = $props();

  let gravatarUrl = $derived(`https://www.gravatar.com/avatar/${artist.email_md5}?d=retro&s=64`);
</script>

<a href="/user/{artist.uid}" class="card card-side bg-base-200 p-3 gap-3 items-center hover:bg-base-300 transition-colors">
  <img
    src={gravatarUrl}
    alt="{artist.username} avatar"
    class="w-12 h-12 rounded-full"
    loading="lazy"
  />
  <div class="flex-1 min-w-0">
    <span class="font-semibold text-base-content truncate block">{artist.username}</span>
    {#if artist.has_about}
      <span class="text-xs opacity-60 truncate block">{@html artist.about_html}</span>
    {/if}
  </div>
</a>
