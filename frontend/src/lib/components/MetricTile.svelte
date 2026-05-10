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

	const bg = $derived(accent === 'emerald' ? 'bg-positive-tint' : 'bg-negative-tint');
	const accentText = $derived(accent === 'emerald' ? 'text-positive' : 'text-negative');
</script>

<div class="rounded-lg {bg} p-3">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">{label}</p>
	<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-ink">{fmt(total)}</p>
	<p class="mt-0.5 font-mono text-[10px] tabular-nums text-ink-muted">
		vs {priorShortLabel}: {fmt(prior)}
	</p>
</div>
