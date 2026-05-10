import { apiFetch, DEV_WORKSPACE } from '$lib/api/client';
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

export type Account = {
	id: string;
	parent_id: string | null;
	name: string;
	account_type: string;
	currency: string;
};

export type EntriesData = {
	entries: Entry[];
	accountNames: Record<string, string>;
};

export async function load(): Promise<JSend<EntriesData>> {
	const [entriesResult, accountsResult] = await Promise.all([
		apiFetch<{ entries: Entry[] }>(`/workspaces/${DEV_WORKSPACE}/entries`),
		apiFetch<{ accounts: Account[] }>(`/workspaces/${DEV_WORKSPACE}/accounts`)
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;

	const accountNames: Record<string, string> = {};
	for (const account of accountsResult.data.accounts) {
		accountNames[account.id] = account.name;
	}

	return {
		status: 'success',
		data: { entries: entriesResult.data.entries, accountNames }
	};
}

export function entryAmount(entry: Entry): number {
	return entry.postings.filter((p) => p.amount > 0).reduce((sum, p) => sum + p.amount, 0);
}

export function formatAmount(cents: number): string {
	const sign = cents < 0 ? '-' : '';
	const abs = Math.abs(cents);
	const dollars = Math.floor(abs / 100);
	const c = (abs % 100).toString().padStart(2, '0');
	return `${sign}$${dollars.toLocaleString()}.${c}`;
}
