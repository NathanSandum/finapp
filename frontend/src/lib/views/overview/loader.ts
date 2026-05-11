import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';
import { fetchAnchors, type Account, type Anchors, type Entry } from '../entries/loader';

export type OverviewRaw = {
	entries: Entry[];
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<OverviewRaw>> {
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${DEV_WORKSPACE}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${DEV_WORKSPACE}/accounts`),
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
