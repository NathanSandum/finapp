import type { JSend } from '$lib/tabs/types';

const PREFIX = import.meta.env.VITE_API_PREFIX ?? '/api';
const TOKEN = import.meta.env.VITE_DEV_TOKEN ?? '';

export const DEV_WORKSPACE = import.meta.env.VITE_DEV_WORKSPACE ?? '';

export async function apiFetch<T>(path: string, init: RequestInit = {}): Promise<JSend<T>> {
	let response: Response;
	try {
		const headers = new Headers(init.headers ?? {});
		if (TOKEN && !headers.has('Authorization')) {
			headers.set('Authorization', `Bearer ${TOKEN}`);
		}
		if (init.body && !headers.has('Content-Type')) {
			headers.set('Content-Type', 'application/json');
		}
		response = await fetch(`${PREFIX}${path}`, { ...init, headers });
	} catch (err) {
		const message = err instanceof Error ? err.message : String(err);
		return { status: 'error', message };
	}

	const body = await response.json().catch(() => null);
	if (body && typeof body === 'object' && 'status' in body) {
		return body as JSend<T>;
	}

	if (!response.ok) {
		return {
			status: 'error',
			message: `${response.status} ${response.statusText || 'request failed'}`,
			code: String(response.status)
		};
	}
	return { status: 'error', message: `malformed response from ${path}` };
}
