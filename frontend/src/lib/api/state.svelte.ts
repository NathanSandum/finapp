import type { AuthUser } from './auth';

const state: { user: AuthUser | null } = $state({ user: null });

export const authState = {
	get user(): AuthUser | null {
		return state.user;
	},
	set(user: AuthUser | null) {
		state.user = user;
	},
	clear() {
		state.user = null;
	}
};

export function initials(name: string): string {
	const parts = name.trim().split(/\s+/);
	if (parts.length === 0 || !parts[0]) return '?';
	if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
	return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
}
