import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import { fetchAnchors, type Account, type Anchors, type Entry } from '../entries/loader';

export type OverviewRaw = {
	entries: Entry[];
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<OverviewRaw>> {
	const id = workspaceState.id();
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${id}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`),
		fetchAnchors()
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	return {
		status: 'success',
		data: {
			entries: entriesResult.data.entries,
			accounts: accountsResult.data.accounts,
			anchors: anchorsResult.data
		}
	};
}
