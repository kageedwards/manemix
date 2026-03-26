// @vitest-environment happy-dom
import { describe, it, expect, vi, beforeEach } from 'vitest';
import fc from 'fast-check';
import { ApiError, apiFetch } from '../client';

describe('API Client Property Tests', () => {
	beforeEach(() => {
		vi.restoreAllMocks();
	});

	/**
	 * Property 18: API error messages are displayed to the user
	 * Validates: Requirements 10.3, 10.6
	 */
	it('Property 18: API errors contain status and message', async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.integer({ min: 400, max: 599 }),
				fc.stringMatching(/^[a-z_]{3,20}$/),
				fc.stringMatching(/^[A-Za-z0-9 ]{1,50}$/),
				async (status, errorCode, message) => {
					const mockResponse = {
						ok: false,
						status,
						statusText: 'Error',
						json: () => Promise.resolve({ error: errorCode, message }),
						headers: new Headers()
					};
					vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(mockResponse as unknown as Response);

					try {
						await apiFetch('/test');
						expect.unreachable('Should have thrown');
					} catch (err) {
						expect(err).toBeInstanceOf(ApiError);
						expect((err as ApiError).status).toBe(status);
						expect((err as ApiError).body.error).toBe(errorCode);
						expect((err as ApiError).message).toBe(message);
					}
				}
			),
			{ numRuns: 50 }
		);
	});

	/**
	 * Property 26: API base URL is configurable
	 * Validates: Requirements 12.5
	 */
	it('Property 26: fetch calls use correct path prefix', async () => {
		await fc.assert(
			fc.asyncProperty(
				fc.stringMatching(/^\/[a-z]{1,20}$/),
				async (path) => {
					const mockResponse = {
						ok: true,
						status: 200,
						type: 'basic',
						json: () => Promise.resolve({}),
						text: () => Promise.resolve('{}'),
						headers: new Headers()
					};
					const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(mockResponse as unknown as Response);

					await apiFetch(path);
					expect(fetchSpy).toHaveBeenCalledTimes(1);
					const calledUrl = fetchSpy.mock.calls[0][0];
					// With empty VITE_API_BASE, the URL should be /api/v1 + path
					expect(calledUrl).toBe(`/api/v1${path}`);
				}
			),
			{ numRuns: 30 }
		);
	});
});
