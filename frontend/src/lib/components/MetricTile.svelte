<script lang="ts">
	import { fmt, pct } from '$lib/format';
	import Num from './Num.svelte';

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

<div class="border-t border-ink px-1 pt-3">
	<p class="text-[10px] font-medium uppercase tracking-[0.18em] {accentText}">{displayLabel}</p>
	<p class="mt-2 font-display text-3xl font-medium tabular-nums leading-none text-ink">
		<Num text={fmt(displayTotal)} />
	</p>
	<div class="mt-3 flex items-baseline justify-between text-[10px] uppercase tracking-[0.12em] text-ink-faint">
		<span>{priorShortLabel}{#if priorVerb}, {priorVerb}{/if} <Num text={fmt(displayPrior)} /></span>
		<span class="font-mono normal-case tracking-normal {arrowClass}">
			<span aria-hidden="true">{arrow}</span> {pct(total, prior)}
		</span>
	</div>
</div>
