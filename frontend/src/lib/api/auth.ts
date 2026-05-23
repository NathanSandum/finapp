import { apiFetch } from './client';
import type { JSend } from '$lib/tabs/types';

export interface AuthUser {
	id: string;
	email: string;
	display_name: string;
}

export interface UserPayload {
	user: AuthUser;
}

export function apiFetchMe(): Promise<JSend<UserPayload>> {
	return apiFetch<UserPayload>('/auth/me');
}

export function apiSignIn(email: string, password: string): Promise<JSend<UserPayload>> {
	return apiFetch<UserPayload>('/auth/login', {
		method: 'POST',
		body: JSON.stringify({ email, password })
	});
}

export function apiLogout(): Promise<JSend<unknown>> {
	return apiFetch('/auth/logout', { method: 'POST' });
}
