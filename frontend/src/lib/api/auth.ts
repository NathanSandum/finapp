import { apiFetch } from './client';

export interface AuthUser {
	id: string;
	email: string;
	display_name: string;
}

interface UserPayload {
	user: AuthUser;
}

export async function fetchMe(): Promise<AuthUser | null> {
	const result = await apiFetch<UserPayload>('/auth/me');
	if (result.status === 'success') return result.data.user;
	return null;
}

export async function devLogin(): Promise<AuthUser | null> {
	const result = await apiFetch<UserPayload>('/auth/dev-login', { method: 'POST' });
	if (result.status === 'success') return result.data.user;
	return null;
}

export async function logout(): Promise<void> {
	await apiFetch('/auth/logout', { method: 'POST' });
}

export async function ensureSignedIn(): Promise<AuthUser | null> {
	const existing = await fetchMe();
	if (existing) return existing;
	return await devLogin();
}
