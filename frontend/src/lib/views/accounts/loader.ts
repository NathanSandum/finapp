import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import type { Account, Anchors, Entry } from '$lib/views/entries/loader';
import type { RecurringRule } from '$lib/views/recurring/loader';

export type AccountsData = {
	accounts: Account[];
	anchors: Anchors;
	entries: Entry[];
	recurringRules: RecurringRule[];
};

export async function load(): Promise<JSend<AccountsData>> {
	const id = workspaceState.id();
	const [accountsResult, anchorsResult, entriesResult, recurringResult] = await Promise.all([
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`),
		apiFetch<Anchors>(`/workspaces/${id}/anchors`),
		apiFetch<{ entries: Entry[] }>(`/workspaces/${id}/entries`),
		apiFetch<{ recurring_rules: RecurringRule[] }>(`/workspaces/${id}/recurring`)
	]);

	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;
	if (entriesResult.status !== 'success') return entriesResult;
	if (recurringResult.status !== 'success') return recurringResult;

	return {
		status: 'success',
		data: {
			accounts: accountsResult.data.accounts,
			anchors: anchorsResult.data,
			entries: entriesResult.data.entries,
			recurringRules: recurringResult.data.recurring_rules
		}
	};
}
