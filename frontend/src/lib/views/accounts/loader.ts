import { entries, accounts, anchors, recurring } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type AccountsReady = { ready: true };

export async function load(): Promise<JSend<AccountsReady>> {
	const [accountsResult, anchorsResult, entriesResult, recurringResult] = await Promise.all([
		accounts.load(),
		anchors.load(),
		entries.load(),
		recurring.load()
	]);
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;
	if (entriesResult.status !== 'success') return entriesResult;
	if (recurringResult.status !== 'success') return recurringResult;
	return { status: 'success', data: { ready: true } };
}
