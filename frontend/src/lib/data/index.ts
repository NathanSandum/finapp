import { apiFetch } from '$lib/api/client';
import { cache } from '$lib/api/cache.svelte';
import { workspaceState } from '$lib/workspace/state.svelte';
import type { JSend } from '$lib/tabs/types';
import { entries } from './entries.svelte';
import { accounts } from './accounts.svelte';
import { anchors } from './anchors.svelte';
import { recurring } from './recurring.svelte';
import type { Entry, Account, Anchors } from './types';
import type { RecurringRule } from './recurring.svelte';

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

export type BootstrapPayload = {
	entries: Entry[];
	accounts: Account[];
	anchors: Anchors;
	recurring_rules: RecurringRule[];
};

/// Fetch every resource in one round-trip and hydrate each resource's reactive
/// state. After this resolves, downstream tab loaders calling .load() get cached
/// values immediately — no further HTTP traffic on boot.
export async function bootstrap(): Promise<JSend<BootstrapPayload>> {
	const result = await apiFetch<BootstrapPayload>(
		`/workspaces/${workspaceState.id()}/bootstrap`
	);
	if (result.status !== 'success') return result;
	entries.hydrate(result.data.entries);
	accounts.hydrate(result.data.accounts);
	anchors.hydrate(result.data.anchors);
	recurring.hydrate(result.data.recurring_rules);
	return result;
}
