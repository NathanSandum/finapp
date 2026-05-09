<script lang="ts">
	import OverviewIcon from './icons/OverviewIcon.svelte';
	import IncomeIcon from './icons/IncomeIcon.svelte';
	import CashFlowIcon from './icons/CashFlowIcon.svelte';
	import BalanceSheetIcon from './icons/BalanceSheetIcon.svelte';
	import EntriesIcon from './icons/EntriesIcon.svelte';

	const icons: Record<string, typeof OverviewIcon> = {
		overview: OverviewIcon,
		income: IncomeIcon,
		'cash-flow': CashFlowIcon,
		'balance-sheet': BalanceSheetIcon,
		entries: EntriesIcon
	};

	let { id, class: klass = 'h-4 w-4' }: { id: string; class?: string } = $props();

	const Icon = $derived(icons[id]);
</script>

{#if Icon}
	<Icon class={klass} />
{/if}
