import { ApiError } from '$lib/api/client';
import { goto } from '$app/navigation';

export function handleError({ error }: { error: unknown }) {
	if (error instanceof ApiError && error.status >= 500) {
		goto('/down');
		return { message: 'Backend unavailable' };
	}
	// Let other errors surface normally
	return { message: error instanceof Error ? error.message : 'Something went wrong' };
}
