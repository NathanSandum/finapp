<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';

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

	const accentText = $derived(
		accent === 'emerald'
			? 'text-positive'
			: accent === 'sky'
				? 'text-investing'
				: 'text-financing'
	);
	const preposition = $derived(total >= 0 ? 'From' : 'To');
	const priorVerb = $derived(prior >= 0 ? verbPositive : verbNegative);
	const arrow = $derived(total >= prior ? '▲' : '▼');
	const arrowClass = $derived('text-ink-muted');
</script>

<div class="mx-auto max-w-xs px-2 py-3">
	<div class="flex items-baseline justify-between">
		<p class="text-[10px] font-medium uppercase tracking-[0.1em] {accentText}">
			{preposition} {noun}
		</p>
		<p class="font-mono text-[10px] tabular-nums {arrowClass}">
			<span aria-hidden="true">{arrow}</span> {pct(total, prior)}
		</p>
	</div>
	<p class="mt-1 font-mono text-xl font-semibold tabular-nums leading-tight text-ink">
		{fmt(Math.abs(total))}
	</p>
	<div class="mt-1 flex justify-between gap-2 font-mono text-[9px] tabular-nums text-ink-faint">
		<span>{priorShortLabel}</span>
		<span>{priorVerb} {fmt(Math.abs(prior))}</span>
	</div>
</div>
