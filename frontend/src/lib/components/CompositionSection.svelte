<script lang="ts">
	import { fmt } from '$lib/views/income/lib';

	type Line = { name: string; amount: number };

	let {
		title,
		total,
		lines,
		base,
		palette,
		minLabelFraction,
		legend = false,
		footnote
	}: {
		title: string;
		total: number;
		lines: Line[];
		base: number;
		palette: string[];
		minLabelFraction: number;
		legend?: boolean;
		footnote?: string;
	} = $props();
</script>

<div>
	<div class="mb-1.5 flex items-baseline justify-between">
		<span class="text-xs font-semibold uppercase tracking-wide text-ink-muted">{title}</span>
		<span class="font-mono text-xs tabular-nums text-ink-muted">{fmt(total)}</span>
	</div>
	<div class="flex h-7 w-full overflow-hidden rounded-md bg-surface-sunken">
		{#each lines as line, i}
			{@const fraction = base > 0 ? line.amount / base : 0}
			<div
				class="{palette[
					i % palette.length
				]} flex items-center justify-center text-[10px] font-medium text-white"
				style:width="{fraction * 100}%"
			>
				{#if fraction > minLabelFraction}{line.name}{/if}
			</div>
		{/each}
	</div>
	{#if legend}
		<ul class="mt-2 grid grid-cols-2 gap-x-6 gap-y-1 text-xs sm:grid-cols-4">
			{#each lines as line, i}
				<li class="flex items-baseline gap-2">
					<span class="h-2 w-2 rounded-sm {palette[i % palette.length]}"></span>
					<span class="text-ink-body">{line.name}</span>
					<span class="ml-auto font-mono tabular-nums text-ink">{fmt(line.amount)}</span>
				</li>
			{/each}
		</ul>
	{/if}
	{#if footnote}
		<p class="mt-1 text-[10px] text-ink-faint">{footnote}</p>
	{/if}
</div>
