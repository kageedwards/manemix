import { writable } from 'svelte/store';
import type { MeResponse } from '$lib/types/index.js';

const DEFAULT_STATE: MeResponse = { logged_in: false };

export const auth = writable<MeResponse>(DEFAULT_STATE);

/**
 * Set the auth state. Called from the root +layout.ts load function
 * after fetching /me/json.
 */
export function setAuth(data: MeResponse): void {
	auth.set(data);
}

/**
 * Log out the current user: POST to /logout, then clear auth state.
 */
export async function logout(): Promise<void> {
	try {
		await fetch('/api/v1/logout', {
			credentials: 'include'
		});
	} catch {
		// Network error — clear state anyway so the UI reflects logged-out
	}
	auth.set({ logged_in: false });
}
