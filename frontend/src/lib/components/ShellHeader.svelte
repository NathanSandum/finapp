<script lang="ts">
	import { onMount } from 'svelte';
	import GranularityPill from './GranularityPill.svelte';
	import PeriodNav from './PeriodNav.svelte';
	import RollingTitle from './RollingTitle.svelte';
	import { tabs } from '$lib/tabs/registry';
	import { tabState } from '$lib/tabs/state.svelte';
	import { periodState } from '$lib/views/period-state.svelte';
	import { periodLabel } from '$lib/views/period';
	import { entryIntent } from '$lib/views/entries/intent.svelte';
	import { accountsSelection } from '$lib/views/accounts/selection.svelte';
	import type { VerbId } from '$lib/views/entries/verbs';
	import type { Account } from '$lib/views/entries/loader';

	function intentForSelection(
		account: Account
	): { verb: VerbId; counterpartyId: string } | null {
		if (account.parent_id === null || account.role === 'cash') return null;
		switch (account.account_type) {
			case 'expense':
				return { verb: 'spent', counterpartyId: account.id };
			case 'revenue':
				return { verb: 'earned', counterpartyId: account.id };
			case 'asset':
				return { verb: 'contributed', counterpartyId: account.id };
			case 'liability':
				return { verb: 'borrowed', counterpartyId: account.id };
			default:
				return null;
		}
	}

	const currentIndex = $derived(Math.max(0, tabs.findIndex((t) => t.id === tabState.get())));
	const current = $derived(tabs[currentIndex] ?? tabs[0]);
	const label = $derived(periodLabel(periodState.year, periodState.month, periodState.granularity));

	const measureEls: (HTMLHeadingElement | undefined)[] = $state(Array(tabs.length).fill(undefined));
	let titleMinWidth = $state(0);

	onMount(() => {
		let max = 0;
		for (const el of measureEls) {
			if (el) max = Math.max(max, el.getBoundingClientRect().width);
		}
		titleMinWidth = Math.ceil(max);
	});

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

<header class="flex shrink-0 items-baseline justify-between gap-3 p-6 pb-0">
	<div class="flex items-baseline gap-3">
		<h3
			class="text-xl font-semibold text-ink"
			style:min-width={titleMinWidth ? `${titleMinWidth}px` : undefined}
		>
			<RollingTitle value={current.label} position={currentIndex} />
		</h3>
		<PeriodNav
			label={label}
			granularity={periodState.granularity}
			onPrev={() => periodState.shift(-1)}
			onNext={() => periodState.shift(1)}
		/>
	</div>
	<div class="title-measure-rack" aria-hidden="true">
		{#each tabs as t, i (t.id)}
			<h3 bind:this={measureEls[i]} class="text-xl font-semibold">{t.label}</h3>
		{/each}
	</div>
	<div class="flex items-baseline gap-3">
		{#if current.newEntryVerb}
			<button
				type="button"
				onclick={newEntry}
				class="bg-surface-inverse px-3 py-1.5 text-xs font-medium uppercase tracking-[0.08em] text-ink-inverse transition-colors hover:bg-surface-inverse-hover"
			>
				+ New
			</button>
		{/if}
		<GranularityPill
			granularity={periodState.granularity}
			onSelect={(g) => periodState.setGranularity(g)}
		/>
	</div>
</header>

<style>
	.title-measure-rack {
		position: absolute;
		left: -9999px;
		top: 0;
		visibility: hidden;
		pointer-events: none;
		white-space: nowrap;
	}
	.title-measure-rack h3 {
		display: inline-block;
	}
</style>
