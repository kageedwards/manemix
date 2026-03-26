import { getUser } from '$lib/api/client';
import { error } from '@sveltejs/kit';
import { ApiError } from '$lib/api/client';
import type { UserProfile } from '$lib/types/index.js';

export async function load({ params }: { params: { uid: string } }): Promise<{ user: UserProfile }> {
	try {
		const user = await getUser(Number(params.uid));
		return { user };
	} catch (err) {
		if (err instanceof ApiError) {
			throw error(err.status, err.message);
		}
		throw error(500, 'Failed to load user profile');
	}
}
