<script lang="ts">
	import { fmt } from '../income/lib';

	let {
		noun,
		total,
		prior,
		priorShortLabel,
		accent,
		verbPositive,
		verbNegative
	}: {
		noun: string;
		total: number;
		prior: number;
		priorShortLabel: string;
		accent: 'emerald' | 'sky' | 'amber';
		verbPositive: string;
		verbNegative: string;
	} = $props();

	const bg = $derived(
		accent === 'emerald'
			? 'bg-emerald-50/50'
			: accent === 'sky'
				? 'bg-sky-50/50'
				: 'bg-amber-50/50'
	);
	const accentText = $derived(
		accent === 'emerald'
			? 'text-emerald-700'
			: accent === 'sky'
				? 'text-sky-700'
				: 'text-amber-700'
	);
	const verb = $derived(total >= 0 ? 'From' : 'To');
	const priorVerb = $derived(prior >= 0 ? verbPositive : verbNegative);
</script>

<div class="rounded-lg {bg} p-3">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">
		<span class="font-bold">{verb}</span> {noun}
	</p>
	<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-neutral-900">
		{fmt(Math.abs(total))}
	</p>
	<p class="mt-0.5 font-mono text-[10px] tabular-nums text-neutral-500">
		vs {priorShortLabel}: {priorVerb} {fmt(Math.abs(prior))}
	</p>
</div>
