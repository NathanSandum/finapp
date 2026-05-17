import { apiFetch } from '$lib/api/client';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';

type Posting = {
	id: string;
	account_id: string;
	amount: number;
	note: string;
};

export type Entry = {
	id: string;
	date: string;
	description: string;
	postings: Posting[];
};

export type AccountRole =
	| 'assets_root'
	| 'liabilities_root'
	| 'equity_root'
	| 'revenue_root'
	| 'expenses_root'
	| 'cash'
	| 'opening_equity';

export type Account = {
	id: string;
	parent_id: string | null;
	name: string;
	account_type: string;
	currency: string;
	role: AccountRole | null;
};

export type Anchors = {
	assets: Account;
	liabilities: Account;
	equity: Account;
	revenue: Account;
	expenses: Account;
	cash: Account;
	opening_equity: Account;
};

export type EntriesData = {
	entries: Entry[];
	accounts: Account[];
	accountNames: Record<string, string>;
	anchors: Anchors;
};

export function fetchAnchors() {
	return apiFetch<Anchors>(`/workspaces/${workspaceState.id()}/anchors`);
}

export async function load(): Promise<JSend<EntriesData>> {
	const id = workspaceState.id();
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${id}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${id}/accounts`),
		fetchAnchors()
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	const accountNames: Record<string, string> = {};
	for (const account of accountsResult.data.accounts) {
		accountNames[account.id] = account.name;
	}

	return {
		status: 'success',
		data: {
			entries: entriesResult.data.entries,
			accounts: accountsResult.data.accounts,
			accountNames,
			anchors: anchorsResult.data
		}
	};
}

export function entryAmount(entry: Entry): number {
	return entry.postings.filter((p) => p.amount > 0).reduce((sum, p) => sum + p.amount, 0);
}
