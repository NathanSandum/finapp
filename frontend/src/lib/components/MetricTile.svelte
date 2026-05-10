<script lang="ts">
	import { fmt } from '$lib/views/income/lib';

	let {
		label,
		total,
		prior,
		priorShortLabel,
		accent
	}: {
		label: string;
		total: number;
		prior: number;
		priorShortLabel: string;
		accent: 'emerald' | 'rose';
	} = $props();

	const accentText = $derived(accent === 'emerald' ? 'text-positive' : 'text-negative');
</script>

<div class="mx-auto flex max-w-xs items-baseline justify-between gap-3 px-1 pt-6 pb-2">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">{label}</p>
	<div class="text-right">
		<p class="font-mono text-xl font-semibold tabular-nums text-ink">{fmt(total)}</p>
		<p class="font-mono text-[10px] tabular-nums text-ink-faint">
			{priorShortLabel} {fmt(prior)}
		</p>
	</div>
</div>
