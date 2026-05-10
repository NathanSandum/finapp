<script lang="ts">
	import { fmt, pct } from '$lib/views/income/lib';

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
		accent: 'emerald' | 'rose' | 'sky' | 'amber';
	} = $props();

	const accentText = $derived(
		accent === 'emerald'
			? 'text-positive'
			: accent === 'rose'
				? 'text-negative'
				: accent === 'sky'
					? 'text-investing'
					: 'text-financing'
	);
	const direction = $derived(
		accent === 'emerald' ? 'higher' : accent === 'rose' ? 'lower' : 'neutral'
	);
	const arrow = $derived(total >= prior ? '▲' : '▼');
	const arrowClass = $derived(
		direction === 'neutral'
			? 'text-ink-muted'
			: (direction === 'higher' ? total >= prior : total <= prior)
				? 'text-positive'
				: 'text-negative'
	);
</script>

<div class="mx-auto max-w-xs px-2 py-3">
	<div class="flex items-baseline justify-between">
		<p class="text-[10px] font-medium uppercase tracking-[0.1em] {accentText}">{label}</p>
		<p class="font-mono text-[10px] tabular-nums {arrowClass}">
			<span aria-hidden="true">{arrow}</span> {pct(total, prior)}
		</p>
	</div>
	<p class="mt-1 font-mono text-xl font-semibold tabular-nums leading-tight text-ink">
		{fmt(total)}
	</p>
	<div class="mt-1 flex justify-between font-mono text-[9px] tabular-nums text-ink-faint">
		<span>{priorShortLabel}</span>
		<span>{fmt(prior)}</span>
	</div>
</div>
