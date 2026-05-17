<script lang="ts">
	import { fmt, pct } from '$lib/format';

	let {
		label,
		total,
		prior,
		priorShortLabel,
		accent,
		signPrefix,
		verbs
	}: {
		label: string;
		total: number;
		prior: number;
		priorShortLabel: string;
		accent: 'emerald' | 'rose' | 'sky' | 'amber';
		signPrefix?: { positive: string; negative: string };
		verbs?: { positive: string; negative: string };
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
	const displayLabel = $derived(
		signPrefix ? `${total >= 0 ? signPrefix.positive : signPrefix.negative} ${label}` : label
	);
	const displayTotal = $derived(signPrefix ? Math.abs(total) : total);
	const priorVerb = $derived(verbs ? (prior >= 0 ? verbs.positive : verbs.negative) : null);
	const displayPrior = $derived(verbs ? Math.abs(prior) : prior);
</script>

<div class="mx-auto max-w-xs px-2 py-3">
	<div class="flex items-baseline justify-between">
		<p class="text-[10px] font-medium uppercase tracking-[0.1em] {accentText}">{displayLabel}</p>
		<p class="font-mono text-[10px] tabular-nums {arrowClass}">
			<span aria-hidden="true">{arrow}</span> {pct(total, prior)}
		</p>
	</div>
	<p class="mt-1 font-mono text-xl font-semibold tabular-nums leading-tight text-ink">
		{fmt(displayTotal)}
	</p>
	<div class="mt-1 flex justify-between gap-2 font-mono text-[9px] tabular-nums text-ink-faint">
		<span>{priorShortLabel}</span>
		<span>{#if priorVerb}{priorVerb}{' '}{/if}{fmt(displayPrior)}</span>
	</div>
</div>
