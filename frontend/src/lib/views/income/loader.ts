import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, Entry } from '../entries/loader';

export type IncomeRaw = {
	entries: Entry[];
	accounts: Account[];
};

export async function load(): Promise<JSend<IncomeRaw>> {
	const id = workspaceState.id();
	const [entriesResult, accountsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${id}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`)
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;

	return {
		status: 'success',
		data: {
			entries: entriesResult.data.entries,
			accounts: accountsResult.data.accounts
		}
	};
}
