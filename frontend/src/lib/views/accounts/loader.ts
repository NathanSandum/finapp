import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, Anchors } from '$lib/views/entries/loader';

export type AccountsData = {
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<AccountsData>> {
	const id = workspaceState.id();
	const [accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`),
		apiFetch<Anchors>(`/workspaces/${id}/anchors`)
	]);

	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	return {
		status: 'success',
		data: {
			accounts: accountsResult.data.accounts,
			anchors: anchorsResult.data
		}
	};
}
