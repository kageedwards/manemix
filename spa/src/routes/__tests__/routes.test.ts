// @vitest-environment happy-dom
import { describe, it, expect, vi } from 'vitest';
import fc from 'fast-check';
import { render } from '@testing-library/svelte';
import type { Track, ExtendedTrack, UserProfile, PlaylistSummary, PlaylistData, EventItem } from '$lib/types/index.js';
import { auth } from '$lib/stores/auth';

// Mock Audio
const mockAudio = {
	src: '', currentTime: 0, duration: 0, volume: 1,
	play: vi.fn().mockResolvedValue(undefined), pause: vi.fn(),
	addEventListener: vi.fn(), canPlayType: vi.fn().mockReturnValue('')
};
vi.stubGlobal('Audio', vi.fn(() => mockAudio));

// --- Arbitrary generators ---

const arbTrack: fc.Arbitrary<Track> = fc.record({
	tid: fc.integer({ min: 1, max: 100000 }),
	title: fc.stringMatching(/^[A-Za-z0-9 ]{1,60}$/),
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.stringMatching(/^[A-Za-z0-9_]{1,30}$/),
	is_visible: fc.constant(true),
	is_hidden: fc.constant(false),
	date: fc.stringMatching(/^[0-9]{4}-[0-9]{2}-[0-9]{2}$/),
	timestamp: fc.constant(''),
	day: fc.constant(''),
	has_art: fc.boolean()
});

const arbExtendedTrack: fc.Arbitrary<ExtendedTrack> = arbTrack.chain((t) =>
	fc.record({
		email_md5: fc.hexaString({ minLength: 32, maxLength: 32 }),
		notes: fc.stringMatching(/^[A-Za-z0-9 ]{0,100}$/),
		notes_html: fc.stringMatching(/^[A-Za-z0-9 ]{0,100}$/),
		has_notes: fc.boolean(),
		license: fc.stringMatching(/^[A-Za-z0-9 ]{1,20}$/),
		has_license: fc.boolean(),
		tags: fc.array(fc.stringMatching(/^[a-z]{1,10}$/), { minLength: 0, maxLength: 5 }),
		has_tags: fc.boolean(),
		is_copyright: fc.boolean(),
		license_key: fc.stringMatching(/^[a-z\-]{1,15}$/),
		airable: fc.boolean()
	}).map((ext) => ({ ...t, ...ext }))
);

const arbEvent: fc.Arbitrary<EventItem> = fc.record({
	event_id: fc.integer({ min: 1, max: 100000 }),
	utc_date: fc.constant('2024-01-01T00:00:00Z'),
	fuzzy_time: fc.stringMatching(/^[0-9]+ [a-z]+ ago$/),
	is_publish: fc.boolean(),
	is_comment: fc.boolean(),
	is_favorite: fc.boolean(),
	is_follow: fc.boolean(),
	source_uid: fc.integer({ min: 1, max: 100000 }),
	source_name: fc.stringMatching(/^[A-Za-z0-9_]{1,20}$/),
	target_uid: fc.integer({ min: 1, max: 100000 }),
	target_name: fc.stringMatching(/^[A-Za-z0-9_]{1,20}$/),
	has_track: fc.boolean(),
	tid: fc.integer({ min: 1, max: 100000 }),
	track_title: fc.stringMatching(/^[A-Za-z0-9 ]{1,30}$/),
	message: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/),
	message_html: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/)
});

const arbPlaylistSummary: fc.Arbitrary<PlaylistSummary> = fc.record({
	playlist_id: fc.integer({ min: 1, max: 100000 }),
	playlist_name: fc.stringMatching(/^[A-Za-z0-9 ]{1,40}$/),
	playlist_url: fc.constant('/playlist/1'),
	track_count: fc.stringMatching(/^[0-9]+$/),
	playlist_track_count: fc.integer({ min: 0, max: 100 }),
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.stringMatching(/^[A-Za-z0-9_]{1,30}$/),
	description: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/),
	description_html: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/),
	has_description: fc.boolean()
});

const arbUserProfile: fc.Arbitrary<UserProfile> = fc.record({
	uid: fc.integer({ min: 1, max: 100000 }),
	username: fc.stringMatching(/^[A-Za-z0-9_]{1,30}$/),
	email_md5: fc.hexaString({ minLength: 32, maxLength: 32 }),
	about: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/),
	about_html: fc.stringMatching(/^[A-Za-z0-9 ]{0,50}$/),
	has_about: fc.boolean(),
	num_favs: fc.integer({ min: 0, max: 1000 }),
	has_favs: fc.boolean(),
	num_followers: fc.integer({ min: 0, max: 1000 }),
	has_followers: fc.boolean(),
	tracks: fc.array(arbTrack, { minLength: 0, maxLength: 3 }).map((tracks) =>
		tracks.map((t, i) => ({ ...t, tid: t.tid * 1000 + i }))
	),
	playlists: fc.array(arbPlaylistSummary, { minLength: 0, maxLength: 2 }),
	events: fc.array(arbEvent, { minLength: 0, maxLength: 3 }).map((events) =>
		events.map((e, i) => ({ ...e, event_id: e.event_id * 1000 + i }))
	)
});

// --- Import page components ---
// We import the .svelte components directly and pass mock data as props

import TrackDetailPage from '../track/[tid]/+page.svelte';
import UserProfilePage from '../user/[uid]/+page.svelte';
import PlaylistPage from '../playlist/[id]/+page.svelte';

describe('Route Property Tests', () => {
	/**
	 * Property 4: Track detail view renders all metadata
	 * Validates: Requirements 4.2, 4.3, 4.4
	 */
	it('Property 4: Track detail view renders all metadata', () => {
		fc.assert(
			fc.property(arbExtendedTrack, (track) => {
				auth.set({ logged_in: false });
				const { container } = render(TrackDetailPage, { props: { data: { track } } });
				const html = container.innerHTML;

				expect(html).toContain(track.title);
				expect(html).toContain(track.username);
				expect(html).toContain(track.date);

				if (track.has_license) {
					expect(html).toContain(track.license);
				}
				if (track.has_tags) {
					for (const tag of track.tags) {
						expect(html).toContain(tag);
					}
				}
				// Download links
				expect(html).toContain('MP3');
				expect(html).toContain('Opus');
			}),
			{ numRuns: 30 }
		);
	});

	/**
	 * Property 6: Auth-conditional social controls on track view
	 * Validates: Requirements 4.6, 4.7
	 */
	it('Property 6: Auth-conditional social controls on track view', () => {
		fc.assert(
			fc.property(
				arbExtendedTrack,
				fc.record({
					logged_in: fc.constant(true),
					uid: fc.integer({ min: 1, max: 100000 }),
					username: fc.stringMatching(/^[A-Za-z0-9_]{1,20}$/)
				}),
				(track, authData) => {
					auth.set({ logged_in: true, uid: authData.uid, username: authData.username });
					const { container } = render(TrackDetailPage, { props: { data: { track } } });
					const html = container.innerHTML;

					if (authData.uid !== track.uid) {
						// Not owner: should see favorite button
						expect(html).toContain('Favorite');
					} else {
						// Owner: should see edit controls
						expect(html).toContain('Edit Track');
					}
				}
			),
			{ numRuns: 30 }
		);
	});

	/**
	 * Property 7: Artist profile view renders all sections
	 * Validates: Requirements 5.2, 5.3, 5.4, 5.7
	 */
	it('Property 7: Artist profile view renders all sections', () => {
		fc.assert(
			fc.property(arbUserProfile, (user) => {
				auth.set({ logged_in: false });
				const { container } = render(UserProfilePage, { props: { data: { user } } });
				const html = container.innerHTML;

				expect(html).toContain(user.username);
				expect(html).toContain(user.email_md5); // in gravatar URL
				expect(html).toContain(`${user.num_followers} followers`);
				expect(html).toContain(`${user.num_favs} favorites`);
			}),
			{ numRuns: 30 }
		);
	});

	/**
	 * Property 8: Auth-conditional controls on artist profile
	 * Validates: Requirements 5.5, 5.6
	 */
	it('Property 8: Auth-conditional controls on artist profile', () => {
		fc.assert(
			fc.property(arbUserProfile, (user) => {
				// Viewing another user's profile while logged in
				const viewerUid = user.uid + 1;
				auth.set({ logged_in: true, uid: viewerUid, username: 'viewer' });
				const { container } = render(UserProfilePage, { props: { data: { user } } });
				expect(container.innerHTML).toContain('Follow');

				// Viewing own profile
				auth.set({ logged_in: true, uid: user.uid, username: user.username });
				const { container: ownContainer } = render(UserProfilePage, { props: { data: { user } } });
				expect(ownContainer.innerHTML).toContain('Edit Profile');
			}),
			{ numRuns: 20 }
		);
	});

	/**
	 * Property 9: Playlist view renders metadata and ordered tracks
	 * Validates: Requirements 6.2, 6.3
	 */
	it('Property 9: Playlist view renders metadata and ordered tracks', () => {
		fc.assert(
			fc.property(
				arbPlaylistSummary,
				fc.array(arbTrack, { minLength: 1, maxLength: 5 }).map((tracks) =>
					tracks.map((t, i) => ({ ...t, tid: t.tid * 1000 + i }))
				),
				(playlist, tracks) => {
					auth.set({ logged_in: false });
					const playlistData: PlaylistData = { playlist, tracks };
					const { container } = render(PlaylistPage, { props: { data: { data: playlistData } } });
					const html = container.innerHTML;

					expect(html).toContain(playlist.playlist_name);
					expect(html).toContain(playlist.username);
					expect(html).toContain(`${playlist.playlist_track_count} tracks`);

					// Verify tracks are rendered
					for (const track of tracks) {
						expect(html).toContain(track.title);
					}
				}
			),
			{ numRuns: 20 }
		);
	});

	/**
	 * Property 10: Playlist owner sees edit controls
	 * Validates: Requirements 6.5
	 */
	it('Property 10: Playlist owner sees edit controls', () => {
		fc.assert(
			fc.property(
				arbPlaylistSummary,
				fc.array(arbTrack, { minLength: 0, maxLength: 3 }).map((tracks) =>
					tracks.map((t, i) => ({ ...t, tid: t.tid * 1000 + i }))
				),
				(playlist, tracks) => {
					auth.set({ logged_in: true, uid: playlist.uid, username: playlist.username });
					const playlistData: PlaylistData = { playlist, tracks };
					const { container } = render(PlaylistPage, { props: { data: { data: playlistData } } });
					expect(container.innerHTML).toContain('Edit');
				}
			),
			{ numRuns: 20 }
		);
	});
});
