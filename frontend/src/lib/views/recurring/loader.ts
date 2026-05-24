import {
	recurring,
	accounts,
	anchors,
	type Account,
	type Anchors,
	type RecurringRule
} from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type { RecurringRule, RecurringTemplate } from '$lib/data';

export type RecurringData = {
	rules: RecurringRule[];
	accounts: Account[];
	anchors: Anchors;
};

export async function load(): Promise<JSend<RecurringData>> {
	const [rulesResult, accountsResult, anchorsResult] = await Promise.all([
		recurring.load(),
		accounts.load(),
		anchors.load()
	]);

	if (rulesResult.status !== 'success') return rulesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;

	return {
		status: 'success',
		data: {
			rules: rulesResult.data,
			accounts: accountsResult.data,
			anchors: anchorsResult.data
		}
	};
}
