import { entries, accounts, anchors, type Account, type Entry, type Anchors } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type EntriesData = {
	entries: Entry[];
	accounts: Account[];
	accountNames: Record<string, string>;
	anchors: Anchors;
};

export async function load(): Promise<JSend<EntriesData>> {
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		entries.load(),
		accounts.load(),
		anchors.load()
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	const accountNames: Record<string, string> = {};
	for (const account of accountsResult.data) {
		accountNames[account.id] = account.name;
	}

	return {
		status: 'success',
		data: {
			entries: entriesResult.data,
			accounts: accountsResult.data,
			accountNames,
			anchors: anchorsResult.data
		}
	};
}
