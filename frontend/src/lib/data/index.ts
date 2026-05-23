import { cache } from '$lib/api/cache.svelte';
import { entries } from './entries.svelte';
import { accounts } from './accounts.svelte';
import { anchors } from './anchors.svelte';
import { recurring } from './recurring.svelte';

export { entries, entryAmount } from './entries.svelte';
export { accounts } from './accounts.svelte';
export { anchors } from './anchors.svelte';
export { recurring, type RecurringRule, type RecurringTemplate } from './recurring.svelte';
export type { Entry, Account, Anchors, AccountRole, Posting } from './types';

export function invalidateAll() {
	entries.invalidate();
	accounts.invalidate();
	anchors.invalidate();
	recurring.invalidate();
	cache.invalidateAll();
}
