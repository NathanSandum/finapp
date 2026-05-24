import { cache } from '$lib/api/cache.svelte';

// Mechanical tabs (entries/accounts/recurring) read live from $lib/data stores,
// so they animate on mutation without any cache reset. Statement tabs hold a
// snapshot of the underlying data inside their tab cache; bust those so the
// next mount recomputes from the updated stores.
const STATEMENT_TABS = ['overview', 'income', 'cash-flow', 'balance-sheet'];

export function invalidateDependentTabs(): void {
	for (const id of STATEMENT_TABS) cache.invalidate(id);
}
