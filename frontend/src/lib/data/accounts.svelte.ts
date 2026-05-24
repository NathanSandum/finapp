import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account } from './types';

const state = $state<{ data: Account[] | null; inFlight: Promise<JSend<Account[]>> | null }>({
	data: null,
	inFlight: null
});

async function fetchAccounts(workspaceId: string): Promise<JSend<Account[]>> {
	const result = await apiFetch<{ accounts: Account[] }>(`/workspaces/${workspaceId}/accounts`);
	if (result.status !== 'success') return result;
	return { status: 'success', data: result.data.accounts };
}

export const accounts = {
	get value(): Account[] | null {
		return state.data;
	},
	async load(): Promise<JSend<Account[]>> {
		if (state.data !== null) return { status: 'success', data: state.data };
		if (state.inFlight) return state.inFlight;
		const promise = fetchAccounts(workspaceState.id()).then((result) => {
			if (result.status === 'success') state.data = result.data;
			state.inFlight = null;
			return result;
		});
		state.inFlight = promise;
		return promise;
	},
	invalidate() {
		state.data = null;
		state.inFlight = null;
	},
	hydrate(value: Account[]) {
		state.data = value;
		state.inFlight = null;
	}
};
