import {
	entries,
	accounts,
	anchors,
	recurring,
	type Account,
	type Anchors,
	type Entry,
	type RecurringRule
} from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type AccountsData = {
	accounts: Account[];
	anchors: Anchors;
	entries: Entry[];
	recurringRules: RecurringRule[];
};

export async function load(): Promise<JSend<AccountsData>> {
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

	return {
		status: 'success',
		data: {
			accounts: accountsResult.data,
			anchors: anchorsResult.data,
			entries: entriesResult.data,
			recurringRules: recurringResult.data
		}
	};
}
