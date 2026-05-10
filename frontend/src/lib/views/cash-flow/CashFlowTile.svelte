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
			? 'bg-positive-tint'
			: accent === 'sky'
				? 'bg-investing-tint'
				: 'bg-financing-tint'
	);
	const accentText = $derived(
		accent === 'emerald'
			? 'text-positive'
			: accent === 'sky'
				? 'text-investing'
				: 'text-financing'
	);
	const verb = $derived(total >= 0 ? 'From' : 'To');
	const priorVerb = $derived(prior >= 0 ? verbPositive : verbNegative);
</script>

<div class="rounded-lg {bg} p-3">
	<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">
		<span class="font-bold">{verb}</span> {noun}
	</p>
	<p class="mt-1 font-mono text-2xl font-semibold tabular-nums text-ink">
		{fmt(Math.abs(total))}
	</p>
	<p class="mt-0.5 font-mono text-[10px] tabular-nums text-ink-muted">
		vs {priorShortLabel}: {priorVerb} {fmt(Math.abs(prior))}
	</p>
</div>
