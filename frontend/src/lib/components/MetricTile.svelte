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

	const bg = $derived(accent === 'emerald' ? 'bg-emerald-50/50' : 'bg-rose-50/50');
	const accentText = $derived(accent === 'emerald' ? 'text-emerald-700' : 'text-rose-700');
</script>

<div class="rounded-lg {bg} p-3">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">{label}</p>
	<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">{fmt(total)}</p>
	<p class="mt-0.5 font-mono text-[10px] tabular-nums text-neutral-500">
		vs {priorShortLabel}: {fmt(prior)}
	</p>
</div>
