import { entries, accounts, type Account, type Entry } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type BalanceSheetRaw = {
	entries: Entry[];
	accounts: Account[];
};

export async function load(): Promise<JSend<BalanceSheetRaw>> {
	const [entriesResult, accountsResult] = await Promise.all([entries.load(), accounts.load()]);

	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;

	return {
		status: 'success',
		data: {
			entries: entriesResult.data,
			accounts: accountsResult.data
		}
	};
}
