<script lang="ts">
	import RankedRows, { type RankedRow } from '$lib/components/RankedRows.svelte';
	import { fmt } from '$lib/format';
	import type { CashFlowSection } from '../compute';

	let {
		operating,
		investing,
		financing
	}: {
		operating: CashFlowSection;
		investing: CashFlowSection;
		financing: CashFlowSection;
	} = $props();

	type SectionView = {
		title: string;
		section: CashFlowSection;
		dot: string;
		bar: string;
		accentText: string;
		verbPositive: string;
		verbNegative: string;
	};

	const sections = $derived<SectionView[]>([
		{
			title: 'Income',
			section: operating,
			dot: 'bg-positive',
			bar: 'bg-positive-line',
			accentText: 'text-positive',
			verbPositive: 'Earned',
			verbNegative: 'Spent'
		},
		{
			title: 'Investing',
			section: investing,
			dot: 'bg-investing',
			bar: 'bg-investing-line',
			accentText: 'text-investing',
			verbPositive: 'Withdrew',
			verbNegative: 'Contributed'
		},
		{
			title: 'Financing',
			section: financing,
			dot: 'bg-financing',
			bar: 'bg-financing-line',
			accentText: 'text-financing',
			verbPositive: 'Borrowed',
			verbNegative: 'Repaid'
		}
	]);

	const signPrefix = { positive: 'From', negative: 'To' };

	function buildRows(section: CashFlowSection, dot: string, bar: string): RankedRow[] {
		return section.lines.map((l) => ({ name: l.name, amount: l.amount, dot, bar }));
	}
</script>

<div class="space-y-4">
	{#each sections as { title, section, dot, bar, accentText, verbPositive, verbNegative }}
		<div>
			<div class="flex h-8 items-center justify-between border-b border-border">
				<p class="text-[9px] font-semibold uppercase tracking-[0.1em] {accentText}">{title}</p>
				<p class="font-mono text-sm font-semibold tabular-nums text-ink">
					<span class="text-[9px] font-semibold uppercase text-ink-muted">
						{section.total >= 0 ? verbPositive : verbNegative}
					</span>
					{fmt(Math.abs(section.total))}
				</p>
			</div>
			{#if section.lines.length === 0}
				<p class="py-1.5 font-mono text-[10px] text-ink-faint">No activity</p>
			{:else}
				<RankedRows rows={buildRows(section, dot, bar)} {signPrefix} />
			{/if}
		</div>
	{/each}
</div>
