import { entries, accounts, anchors } from '$lib/data';
import type { JSend } from '$lib/tabs/types';

export type EntriesReady = { ready: true };

/// Ensures the reactive stores are hydrated. The view reads live from
/// $lib/data; this loader only blocks the tab until the first fetch lands.
export async function load(): Promise<JSend<EntriesReady>> {
	const [entriesResult, accountsResult, anchorsResult] = await Promise.all([
		entries.load(),
		accounts.load(),
		anchors.load()
	]);
	if (entriesResult.status !== 'success') return entriesResult;
	if (accountsResult.status !== 'success') return accountsResult;
	if (anchorsResult.status !== 'success') return anchorsResult;
	return { status: 'success', data: { ready: true } };
}
