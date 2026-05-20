<script lang="ts">
	import GranularityPill from './GranularityPill.svelte';
	import PeriodNav from './PeriodNav.svelte';
	import SolidButton from './SolidButton.svelte';
	import WorkspaceSwitcher from '$lib/workspace/WorkspaceSwitcher.svelte';
	import UserMenu from './UserMenu.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { workspaceState } from '$lib/workspace/state.svelte';
	import { authState, initials } from '$lib/api/state.svelte';
	import { periodState } from '$lib/views/period-state.svelte';
	import { periodLabel } from '$lib/views/period';
	import { entryIntent } from '$lib/views/entries/intent.svelte';
	import { accountsSelection } from '$lib/views/accounts/selection.svelte';
	import { getDiscrete, setDiscrete } from '$lib/ui/discrete.svelte';
	import type { VerbId } from '$lib/views/entries/verbs';
	import type { Account } from '$lib/views/entries/loader';

	function intentForSelection(account: Account): { verb: VerbId; counterpartyId: string } | null {
		if (account.role === 'cash') return null;
		const isRoot = account.parent_id === null;
		const counterpartyId = isRoot ? '' : account.id;
		switch (account.account_type) {
			case 'expense':
				return { verb: 'spent', counterpartyId };
			case 'revenue':
				return { verb: 'earned', counterpartyId };
			case 'asset':
				return { verb: 'contributed', counterpartyId };
			case 'liability':
				return { verb: 'borrowed', counterpartyId };
			default:
				return null;
		}
	}

	let switcherOpen = $state(false);
	let userMenuOpen = $state(false);

	const current = $derived(tabs.find((t) => t.id === tabState.get()) ?? tabs[0]);
	const label = $derived(periodLabel(periodState.year, periodState.month, periodState.granularity));
	const activeName = $derived(workspaceState.active?.name ?? '');
	const displayName = $derived(authState.user?.display_name ?? '');
	const userInitials = $derived(displayName ? initials(displayName) : '');
	const discreteOn = $derived(getDiscrete());

	function newEntry() {
		const defaultVerb = current.newEntryVerb;
		if (!defaultVerb) return;
		const selection = current.id === 'accounts' ? accountsSelection.get() : null;
		const override = selection ? intentForSelection(selection) : null;
		entryIntent.set({
			verb: override?.verb ?? defaultVerb,
			counterpartyId: override?.counterpartyId,
			sourceTab: current.id
		});
		tabState.set('entries');
	}
</script>

<header class="shrink-0 border-b-2 border-ink bg-surface">
	<!-- top row: brand, workspace, period controls, user -->
	<div class="flex items-center gap-6 px-8 py-3">
		<div class="flex items-baseline gap-3">
			<span class="font-display text-xl font-semibold tracking-tight text-ink">finapp</span>
			<span class="text-[10px] uppercase tracking-[0.2em] text-ink-faint">est. 2026</span>
		</div>

		<div class="relative">
			<button
				type="button"
				onclick={() => (switcherOpen = !switcherOpen)}
				class="flex items-center gap-2 border border-border bg-surface-card px-2.5 py-1 text-xs hover:border-ink"
				aria-haspopup="menu"
				aria-expanded={switcherOpen}
			>
				<span class="text-[10px] uppercase tracking-[0.18em] text-ink-faint">Ledger</span>
				<span class="font-display text-sm text-ink">{activeName || '—'}</span>
				<svg class="h-3 w-3 text-ink-muted {switcherOpen ? 'rotate-180' : ''}" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
					<path fill-rule="evenodd" d="M5 8l5 5 5-5H5z" clip-rule="evenodd" />
				</svg>
			</button>
			{#if switcherOpen}
				<WorkspaceSwitcher onClose={() => (switcherOpen = false)} />
			{/if}
		</div>

		<div class="ml-auto flex items-center gap-5">
			<PeriodNav
				{label}
				granularity={periodState.granularity}
				year={periodState.year}
				month={periodState.month}
				isAtCurrent={periodState.isAtCurrent}
				onPrev={() => periodState.shift(-1)}
				onNext={() => periodState.shift(1)}
				onGoToCurrent={() => periodState.goToCurrent()}
			/>
			<GranularityPill
				granularity={periodState.granularity}
				onSelect={(g) => periodState.setGranularity(g)}
			/>

			{#if current.newEntryVerb}
				<SolidButton size="sm" onclick={newEntry}>+ Record</SolidButton>
			{/if}

			<button
				type="button"
				onclick={() => setDiscrete(!discreteOn)}
				class="text-ink-muted hover:text-ink"
				aria-pressed={discreteOn}
				aria-label={discreteOn ? 'Show amounts' : 'Hide amounts'}
				title={discreteOn ? 'Show amounts' : 'Hide amounts'}
			>
				<svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
					{#if discreteOn}
						<path d="M2.5 10c1.5-2.5 4.3-5 7.5-5 1.4 0 2.7.5 3.8 1.2L4.3 16.5 3 15.2zM17.5 10c-1.5 2.5-4.3 5-7.5 5-1 0-2-.2-2.9-.6l9.7-9.7c.2.4.5.8.7 1.3z" />
					{:else}
						<path d="M10 5c-4.2 0-7 4-7.8 5C3 11 5.8 15 10 15s7-4 7.8-5C17 9 14.2 5 10 5zm0 8a3 3 0 110-6 3 3 0 010 6z" />
					{/if}
				</svg>
			</button>

			<div class="relative">
				<button
					type="button"
					onclick={() => (userMenuOpen = !userMenuOpen)}
					class="flex h-7 w-7 items-center justify-center bg-gradient-to-br from-avatar-from to-avatar-to text-[10px] font-semibold uppercase tracking-wider text-ink-inverse"
					aria-haspopup="menu"
					aria-expanded={userMenuOpen}
					aria-label={displayName || 'User menu'}
				>
					{userInitials || '·'}
				</button>
				{#if userMenuOpen}
					<UserMenu onClose={() => (userMenuOpen = false)} />
				{/if}
			</div>
		</div>
	</div>

	<!-- second row: tabs -->
	<nav class="flex items-center gap-7 px-8 pb-0 pt-1 text-[11px] uppercase tracking-[0.18em]">
		{#each tabs as tab, i (tab.id)}
			{@const groupChanged = i > 0 && tab.group !== tabs[i - 1].group}
			{#if groupChanged}
				<span class="text-ink-faint" aria-hidden="true">·</span>
			{/if}
			<button
				type="button"
				onclick={() => tabState.set(tab.id)}
				class="relative pb-2.5 pt-1 transition-colors {current.id === tab.id
					? 'text-ink'
					: 'text-ink-muted hover:text-ink'}"
				aria-current={current.id === tab.id ? 'page' : undefined}
			>
				{tab.label}
				{#if current.id === tab.id}
					<span
						class="absolute -bottom-[2px] left-0 right-0 h-[2px] bg-ink"
						aria-hidden="true"
					></span>
				{/if}
			</button>
		{/each}
	</nav>
</header>
