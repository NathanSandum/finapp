import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';
import type { Account, Entry } from '../entries/loader';

export type CashFlowRaw = {
	entries: Entry[];
	accounts: Account[];
};

export async function load(): Promise<JSend<CashFlowRaw>> {
	const [entriesResult, accountsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${DEV_WORKSPACE}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${DEV_WORKSPACE}/accounts`)
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
