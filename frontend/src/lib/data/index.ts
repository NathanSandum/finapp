import { cache } from '$lib/api/cache.svelte';
import { entries } from './entries.svelte';
import { accounts } from './accounts.svelte';
import { anchors } from './anchors.svelte';
import { recurring } from './recurring.svelte';
import * as bootstrapBackend from '$lib/acl/bootstrap';
import type { JSend } from '$lib/tabs/types';

export { entries, entryAmount } from './entries.svelte';
export { accounts } from './accounts.svelte';
export { anchors } from './anchors.svelte';
export { recurring } from './recurring.svelte';
export type {
	Entry,
	Account,
	Anchors,
	AccountRole,
	Posting,
	RecurringRule,
	RecurringTemplate
} from './types';

/// Wipe all domain stores + tab caches. Used on logout / workspace switch.
export function clearAll(): void {
	entries.clear();
	accounts.clear();
	anchors.clear();
	recurring.clear();
	cache.invalidateAll();
}

/// Fetch every resource in one round-trip and hydrate the reactive stores so
/// downstream tab loaders see cached values immediately.
export async function bootstrap(): Promise<JSend<bootstrapBackend.Bundle>> {
	const result = await bootstrapBackend.fetchAll();
	if (result.status !== 'success') return result;
	entries.hydrate(result.data.entries);
	accounts.hydrate(result.data.accounts);
	anchors.hydrate(result.data.anchors);
	recurring.hydrate(result.data.recurringRules);
	return result;
}
