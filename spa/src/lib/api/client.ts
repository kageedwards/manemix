import type {
	ExtendedTrack,
	UserProfile,
	PlaylistData,
	PlaylistSummary,
	AlbumData,
	AlbumSummary,
	MeResponse,
	Track,
	Artist,
	PlaybackContext
} from '$lib/types/index.js';

const API_BASE = import.meta.env.VITE_API_BASE ?? '';
const API_PREFIX = '/api/v1';

export class ApiError extends Error {
	status: number;
	body: { error: string; message?: string };

	constructor(status: number, body: { error: string; message?: string }) {
		super(body.message ?? body.error ?? `HTTP ${status}`);
		this.name = 'ApiError';
		this.status = status;
		this.body = body;
	}
}

export async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
	const res = await fetch(`${API_BASE}${API_PREFIX}${path}`, {
		credentials: 'include',
		headers: { Accept: 'application/json', ...init?.headers },
		...init
	});
	if (!res.ok) {
		const err = await res.json().catch(() => ({ error: res.statusText }));
		throw new ApiError(res.status, err);
	}
	const text = await res.text();
	if (!text) return { ok: true } as T;
	try {
		return JSON.parse(text) as T;
	} catch {
		// Response was not JSON (e.g. redirect followed to HTML page)
		return { ok: true } as T;
	}
}

// Typed endpoint functions (paths are relative to /api/v1)
export const getTrack = (tid: number) => apiFetch<ExtendedTrack>(`/track/${tid}`);
export const getUser = (uid: number) => apiFetch<UserProfile>(`/user/${uid}`);
export const getPlaylist = (id: number) => apiFetch<PlaylistData>(`/playlist/${id}`);
export const getMe = () => apiFetch<MeResponse>(`/me`);
export const getLatestTracks = () => apiFetch<Track[]>(`/tracks/latest`);
export const getFeaturedTracks = () => apiFetch<Track[]>(`/tracks/featured`);
export const getRandomTracks = () => apiFetch<Track[]>(`/tracks/random`);
export const searchTracks = (q: string) =>
	apiFetch<Track[]>(`/tracks/search?q=${encodeURIComponent(q)}`);
export const getArtists = () => apiFetch<Artist[]>(`/artists`);
export const searchArtists = (q: string) =>
	apiFetch<Artist[]>(`/users/search?q=${encodeURIComponent(q)}`);
export const getFavorites = (uid: number) =>
	apiFetch<Track[]>(`/user/${uid}/favorites`);
export const getPublicPlaylists = () => apiFetch<PlaylistSummary[]>(`/playlists`);

// Playlist management
export const createPlaylist = (name: string, description = '', isPublic = false) =>
	apiFetch<{ ok: boolean; id: number }>(`/playlist/new`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ name, description, public: isPublic })
	});
export const editPlaylist = (id: number, data: { name?: string; description?: string; public?: boolean }) =>
	apiFetch(`/playlist/${id}/edit`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
export const deletePlaylist = (id: number) =>
	apiFetch(`/playlist/${id}/delete`, { method: 'POST' });
export const addTrackToPlaylist = (playlistId: number, trackId: number) =>
	apiFetch(`/playlist/${playlistId}/add`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ track_id: trackId })
	});
export const removeTrackFromPlaylist = (playlistId: number, trackId: number) =>
	apiFetch(`/playlist/${playlistId}/remove`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ track_id: trackId })
	});

// Social actions (JSON body, JSON response)
export const favoriteTrack = (tid: number) =>
	apiFetch(`/track/${tid}/favorite`, { method: 'POST' });
export const unfavoriteTrack = (tid: number) =>
	apiFetch(`/track/${tid}/unfavorite`, { method: 'POST' });
export const followUser = (uid: number) =>
	apiFetch(`/user/${uid}/follow`, { method: 'POST' });
export const unfollowUser = (uid: number) =>
	apiFetch(`/user/${uid}/unfollow`, { method: 'POST' });
export const postComment = (target: string, id: number, msg: string, name?: string) =>
	apiFetch(`/${target}/${id}/comment`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ msg, name })
	});

// Playback queue
export const getNextTracks = (currentTid: number, ctx: PlaybackContext) =>
	apiFetch<Track[]>(`/queue/next`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ current_tid: currentTid, ...ctx })
	});

// Album management
export const getPublicAlbums = () => apiFetch<AlbumSummary[]>(`/albums`);
export const getAlbum = (id: number) => apiFetch<AlbumData>(`/album/${id}`);
export const createAlbum = (name: string, description = '') =>
	apiFetch<{ ok: boolean; id: number }>(`/album/new`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ name, description })
	});
export const editAlbum = (id: number, data: { name?: string; description?: string }) =>
	apiFetch(`/album/${id}/edit`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(data)
	});
export const publishAlbum = (id: number) =>
	apiFetch(`/album/${id}/publish`, { method: 'POST' });
export const unpublishAlbum = (id: number) =>
	apiFetch(`/album/${id}/unpublish`, { method: 'POST' });
export const deleteAlbum = (id: number) =>
	apiFetch(`/album/${id}/delete`, { method: 'POST' });
export const addTrackToAlbum = (albumId: number, trackId: number) =>
	apiFetch(`/album/${albumId}/add`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ track_id: trackId })
	});
export const removeTrackFromAlbum = (albumId: number, trackId: number) =>
	apiFetch(`/album/${albumId}/remove`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ track_id: trackId })
	});
