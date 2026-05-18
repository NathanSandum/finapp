import type { Account } from '$lib/views/entries/loader';

const state: { current: Account | null } = $state({ current: null });

export const accountsSelection = {
	get(): Account | null {
		return state.current;
	},
	set(account: Account | null) {
		state.current = account;
	}
};
