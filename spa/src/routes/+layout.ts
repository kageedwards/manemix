import { getMe, ApiError } from '$lib/api/client';
import { redirect } from '@sveltejs/kit';
import type { MeResponse } from '$lib/types/index.js';

export const prerender = false;
export const ssr = false;

export async function load({ url }: { url: URL }): Promise<{ auth: MeResponse }> {
	// Don't call the backend when we're already on the /down page
	if (url.pathname === '/down') {
		return { auth: { logged_in: false } };
	}

	try {
		const auth = await getMe();
		return { auth };
	} catch (err) {
		// Network TypeError or 502/503/504 = backend is down
		const isGatewayError = err instanceof ApiError && err.status >= 500;
		const isNetworkError = !(err instanceof ApiError);
		if (isGatewayError || isNetworkError) {
			redirect(307, '/down');
		}
		// Any other ApiError = backend responded fine, just not logged in
		return { auth: { logged_in: false } };
	}
}
