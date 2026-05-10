<script lang="ts">
	import { fmt } from '../../income/lib';
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
		accentText: string;
		verbPositive: string;
		verbNegative: string;
	};

	const sections = $derived<SectionView[]>([
		{
			title: 'Income',
			section: operating,
			accentText: 'text-emerald-700',
			verbPositive: 'Earned',
			verbNegative: 'Spent'
		},
		{
			title: 'Investing',
			section: investing,
			accentText: 'text-sky-700',
			verbPositive: 'Withdrew',
			verbNegative: 'Contributed'
		},
		{
			title: 'Financing',
			section: financing,
			accentText: 'text-amber-700',
			verbPositive: 'Borrowed',
			verbNegative: 'Repaid'
		}
	]);
</script>

<div class="grid grid-cols-1 gap-3 md:grid-cols-3">
	{#each sections as { title, section, accentText, verbPositive, verbNegative }}
		<div class="rounded-xl border border-neutral-200 bg-white p-4">
			<div class="mb-2 flex items-baseline justify-between">
				<h4 class="text-xs font-semibold uppercase tracking-wide {accentText}">{title}</h4>
				<span class="font-mono text-base font-semibold tabular-nums text-neutral-900">
					<span class="text-xs font-bold text-neutral-500">
						{section.total >= 0 ? verbPositive : verbNegative}
					</span>
					{fmt(Math.abs(section.total))}
				</span>
			</div>
			{#if section.lines.length === 0}
				<p class="font-mono text-[11px] text-neutral-400">No activity</p>
			{:else}
				<ul class="space-y-1.5">
					{#each section.lines as line}
						<li class="flex items-baseline justify-between text-xs">
							<span class="text-neutral-700">
								<span class="text-neutral-400">{line.amount >= 0 ? 'From' : 'To'}</span>
								{line.name}
							</span>
							<span class="font-mono tabular-nums text-neutral-900">
								{fmt(Math.abs(line.amount))}
							</span>
						</li>
					{/each}
				</ul>
			{/if}
		</div>
	{/each}
</div>
