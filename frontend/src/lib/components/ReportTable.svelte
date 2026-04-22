<script lang="ts">
	import type { MultiPeriodRow } from '$lib/types';
	import { formatAmount } from '$lib/format';

	type Section = {
		title: string;
		rows: MultiPeriodRow[];
		totals: number[];
	};

	type BottomLine = {
		label: string;
		values: number[];
	};

	let {
		periods,
		sections,
		bottomLine
	}: {
		periods: string[];
		sections: Section[];
		bottomLine?: BottomLine;
	} = $props();

	function amountColor(cents: number): string {
		if (cents > 0) return 'text-emerald-400';
		if (cents < 0) return 'text-red-400';
		return 'text-gray-600';
	}
</script>

<table class="w-full border-collapse text-xs">
	<thead>
		<tr class="border-b border-gray-800">
			<th class="w-48 py-1.5 pl-3 pr-2 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-600">
				Account
			</th>
			{#each periods as period}
				<th class="px-2 py-1.5 text-right text-[10px] font-semibold uppercase tracking-wider text-gray-600">
					{period}
				</th>
			{/each}
		</tr>
	</thead>
	<tbody>
		{#each sections as section, sectionIndex}
			<!-- Section header -->
			<tr>
				<td
					colspan={periods.length + 1}
					class="pt-3 pb-1 pl-3 text-[10px] font-semibold uppercase tracking-wider text-gray-500"
					class:pt-4={sectionIndex > 0}
				>
					{section.title}
				</td>
			</tr>

			<!-- Account rows -->
			{#each section.rows as row}
				<tr class="hover:bg-gray-900/50">
					<td class="py-1 pl-6 pr-2 text-gray-300">{row.name}</td>
					{#each row.balances as balance}
						<td class="px-2 py-1 text-right font-medium tabular-nums {amountColor(balance)}">
							{formatAmount(balance)}
						</td>
					{/each}
				</tr>
			{/each}

			<!-- Section total -->
			<tr class="border-t border-gray-800">
				<td class="py-1.5 pl-3 pr-2 font-semibold text-gray-200">Total {section.title}</td>
				{#each section.totals as total}
					<td class="px-2 py-1.5 text-right font-semibold tabular-nums {amountColor(total)}">
						{formatAmount(total)}
					</td>
				{/each}
			</tr>
		{/each}

		<!-- Bottom line -->
		{#if bottomLine}
			<tr class="border-t-2 border-gray-700">
				<td class="py-2 pl-3 pr-2 font-bold text-gray-100">{bottomLine.label}</td>
				{#each bottomLine.values as value}
					<td class="px-2 py-2 text-right font-bold tabular-nums {amountColor(value)}">
						{formatAmount(value)}
					</td>
				{/each}
			</tr>
		{/if}
	</tbody>
</table>
