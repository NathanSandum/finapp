import { recurring, accounts, anchors } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type RecurringReady = { ready: true };

export async function load(): Promise<JSend<RecurringReady>> {
	const [rulesResult, accountsResult, anchorsResult] = await Promise.all([
		recurring.load(),
		accounts.load(),
		anchors.load()
	]);
	if (rulesResult.status !== 'success') return rulesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;
	return { status: 'success', data: { ready: true } };
}
