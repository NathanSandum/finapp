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

<table class="w-full border-collapse font-mono text-xs">
	<!-- Column headers -->
	<thead>
		<tr>
			<th
				class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-left text-[10px] font-semibold uppercase tracking-wider text-gray-400"
			></th>
			{#each periods as period}
				<th
					class="border border-gray-700 bg-gray-900 px-2 py-1.5 text-right text-[10px] font-semibold uppercase tracking-wider text-gray-400"
				>
					{period}
				</th>
			{/each}
		</tr>
	</thead>
	<tbody>
		{#each sections as section}
			<!-- Section title row -->
			<tr>
				<td class="border border-gray-700 bg-gray-800/50 px-2 py-1 font-bold text-gray-200">
					{section.title}
				</td>
				{#each periods as _}
					<td class="border border-gray-700 bg-gray-800/50"></td>
				{/each}
			</tr>

			<!-- Account rows -->
			{#each section.rows as row}
				<tr>
					<td class="border border-gray-700 py-1 pl-4 pr-2 text-gray-300">
						{row.name}
					</td>
					{#each row.balances as balance}
						<td
							class="border border-gray-700 px-2 py-1 text-right tabular-nums {amountColor(balance)}"
						>
							{formatAmount(balance)}
						</td>
					{/each}
				</tr>
			{/each}

			<!-- Section total row -->
			<tr>
				<td class="border border-gray-700 bg-gray-800/30 px-2 py-1 font-bold text-gray-200">
					Total {section.title}
				</td>
				{#each section.totals as total}
					<td
						class="border border-gray-700 bg-gray-800/30 px-2 py-1 text-right font-bold tabular-nums {amountColor(total)}"
					>
						{formatAmount(total)}
					</td>
				{/each}
			</tr>

			<!-- Blank separator row -->
			<tr>
				<td class="border border-gray-700 py-1"></td>
				{#each periods as _}
					<td class="border border-gray-700 py-1"></td>
				{/each}
			</tr>
		{/each}

		<!-- Bottom line -->
		{#if bottomLine}
			<tr>
				<td class="border-2 border-gray-600 bg-gray-800 px-2 py-1.5 font-bold text-gray-100">
					{bottomLine.label}
				</td>
				{#each bottomLine.values as value}
					<td
						class="border-2 border-gray-600 bg-gray-800 px-2 py-1.5 text-right font-bold tabular-nums {amountColor(value)}"
					>
						{formatAmount(value)}
					</td>
				{/each}
			</tr>
		{/if}
	</tbody>
</table>
