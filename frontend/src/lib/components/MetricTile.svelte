<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';

	let {
		label,
		total,
		prior,
		priorShortLabel,
		accent,
		direction
	}: {
		label: string;
		total: number;
		prior: number;
		priorShortLabel: string;
		accent: 'emerald' | 'rose';
		direction: 'higher' | 'lower';
	} = $props();

	const positive = $derived(direction === 'higher' ? total >= prior : total <= prior);
	const bg = $derived(accent === 'emerald' ? 'bg-emerald-50/50' : 'bg-rose-50/50');
	const accentText = $derived(accent === 'emerald' ? 'text-emerald-700' : 'text-rose-700');
</script>

<div class="rounded-lg {bg} p-3">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">{label}</p>
	<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">{fmt(total)}</p>
	<p
		class="mt-0.5 font-mono text-[10px] tabular-nums {positive
			? 'text-emerald-700'
			: 'text-rose-700'}"
	>
		{pct(total, prior)} vs {priorShortLabel}
	</p>
</div>
