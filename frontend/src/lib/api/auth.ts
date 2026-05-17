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

export function fetchMe(): Promise<JSend<UserPayload>> {
	return apiFetch<UserPayload>('/auth/me');
}

export function signIn(email: string, password: string): Promise<JSend<UserPayload>> {
	return apiFetch<UserPayload>('/auth/login', {
		method: 'POST',
		body: JSON.stringify({ email, password })
	});
}

export function logout(): Promise<JSend<unknown>> {
	return apiFetch('/auth/logout', { method: 'POST' });
}
