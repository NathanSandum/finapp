import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
import type { JSend } from '$lib/tabs/types';
import type { Account, Anchors } from '$lib/views/entries/loader';

export type AccountsData = {
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<AccountsData>> {
	const [accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ accounts: Account[] }>(`/workspaces/${DEV_WORKSPACE}/accounts`),
		apiFetch<Anchors>(`/workspaces/${DEV_WORKSPACE}/anchors`)
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
