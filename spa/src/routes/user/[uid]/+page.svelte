<script lang="ts">
  import { tt } from '$lib/i18n';
  import { auth } from '$lib/stores/auth';
  import TrackList from '$lib/components/TrackList.svelte';
  import PlaylistCard from '$lib/components/PlaylistCard.svelte';
  import AlbumCard from '$lib/components/AlbumCard.svelte';
  import EventList from '$lib/components/EventList.svelte';
  import FollowButton from '$lib/components/FollowButton.svelte';
  import CommentForm from '$lib/components/CommentForm.svelte';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();
  let user = $derived(data.user);
  let isOwnProfile = $derived($auth.logged_in && $auth.uid === user.uid);
  let visiblePlaylists = $derived(
    isOwnProfile ? user.playlists : user.playlists.filter(p => p.is_public)
  );
  let visibleAlbums = $derived(
    isOwnProfile ? (user.albums ?? []) : (user.albums ?? []).filter(a => a.is_public)
  );
  let gravatarUrl = $derived(`https://www.gravatar.com/avatar/${user.email_md5}?d=retro&s=128`);
</script>

<svelte:head>
  <title>{user.username} — {$tt('site_name')}</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <!-- Profile header -->
  <div class="flex gap-4 items-start">
    <img src={gravatarUrl} alt="{user.username} avatar" class="w-24 h-24 rounded-full" />
    <div class="flex-1">
      <h1 class="text-2xl font-bold">{user.username}</h1>
      <div class="text-sm opacity-70 flex gap-4 mt-1">
        <span>{user.num_followers} followers</span>
        <span>{user.num_favs} favorites</span>
      </div>
      <div class="flex gap-2 mt-3">
        {#if $auth.logged_in && !isOwnProfile}
          <FollowButton uid={user.uid} />
        {/if}
        {#if isOwnProfile}
          <a href="/account" class="btn btn-sm btn-ghost">Edit Profile</a>
          <a href="/upload" class="btn btn-sm btn-ghost">Upload Track</a>
        {/if}
      </div>
    </div>
  </div>

  <!-- Bio -->
  {#if user.has_about}
    <div class="prose prose-sm max-w-none">{@html user.about_html}</div>
  {/if}

  <!-- Tracks -->
  {#if user.tracks.length > 0}
    <section>
      <h2 class="text-lg font-bold mb-3">Tracks</h2>
      <TrackList tracks={user.tracks} playbackContext={{ context: 'user', param: String(user.uid) }} />
    </section>
  {/if}

  <!-- Playlists -->
  {#if visiblePlaylists.length > 0}
    <section>
      <h2 class="text-lg font-bold mb-3">Playlists</h2>
      <div class="flex flex-col gap-2">
        {#each visiblePlaylists as playlist}
          <PlaylistCard {playlist} />
        {/each}
      </div>
    </section>
  {/if}

  <!-- Albums -->
  {#if visibleAlbums.length > 0}
    <section>
      <h2 class="text-lg font-bold mb-3">Albums</h2>
      <div class="flex flex-col gap-2">
        {#each visibleAlbums as album}
          <AlbumCard {album} />
        {/each}
      </div>
    </section>
  {/if}

  <!-- Favorites link -->
  {#if user.has_favs}
    <a href="/user/{user.uid}/favorites" class="btn btn-sm btn-ghost self-start">
      View Favorites ({user.num_favs})
    </a>
  {/if}

  <!-- Activity -->
  {#if user.events.length > 0}
    <section>
      <h2 class="text-lg font-bold mb-3">Recent Activity</h2>
      <EventList events={user.events} />
    </section>
  {/if}

  <!-- Comments -->
  <section>
    <h3 class="text-sm font-semibold mb-2">Leave a comment</h3>
    <CommentForm target="user" id={user.uid} />
  </section>
</div>
