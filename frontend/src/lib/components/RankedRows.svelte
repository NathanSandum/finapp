<script lang="ts" module>
	export type RankedRow = {
		name: string;
		amount: number;
		dot: string;
		bar: string;
	};
</script>

<script lang="ts">
	import { fmt } from '$lib/format';
	import Num from './Num.svelte';

	let {
		rows,
		signPrefix
	}: {
		rows: RankedRow[];
		signPrefix?: { positive: string; negative: string };
	} = $props();

	const max = $derived(Math.max(...rows.map((r) => Math.abs(r.amount)), 1));
</script>

<ul>
	{#each rows as row (row.name)}
		{@const f = Math.abs(row.amount) / max}
		{@const prefix = signPrefix
			? row.amount >= 0
				? signPrefix.positive
				: signPrefix.negative
			: null}
		<li class="grid h-8 grid-cols-[8px_1fr_auto] items-center gap-2 border-b border-border-faint">
			<span class="h-1.5 w-1.5 {row.dot}"></span>
			<div class="min-w-0">
				<div class="flex items-baseline justify-between text-[11px]">
					<span class="truncate text-ink-body">
						{#if prefix}<span class="text-ink-faint">{prefix}</span>{' '}{/if}{row.name}
					</span>
				</div>
				<div class="mt-0.5 h-[2px] w-full bg-surface-sunken">
					<div class="bar h-full {row.bar}" style:width="{f * 100}%"></div>
				</div>
			</div>
			<span class="font-mono text-[11px] tabular-nums text-ink"><Num text={fmt(Math.abs(row.amount))} /></span>
		</li>
	{/each}
</ul>

<style>
	.bar {
		transition: width 520ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	@media (prefers-reduced-motion: reduce) {
		.bar {
			transition: none;
		}
	}
</style>
