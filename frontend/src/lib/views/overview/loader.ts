import { entries, accounts, anchors, type Account, type Anchors, type Entry } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type OverviewRaw = {
	entries: Entry[];
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<OverviewRaw>> {
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		entries.load(),
		accounts.load(),
		anchors.load()
	]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	return {
		status: 'success',
		data: {
			entries: entriesResult.data,
			accounts: accountsResult.data,
			anchors: anchorsResult.data
		}
	};
}
