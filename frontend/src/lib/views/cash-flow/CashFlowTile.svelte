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

<div class="border-t border-border">
	<div class="mx-auto flex max-w-xs items-baseline justify-between gap-3 px-1 py-2">
		<p class="text-[10px] font-medium uppercase tracking-wide {accentText}">
			{verb} {noun}
		</p>
		<div class="text-right">
			<p class="font-mono text-xl font-semibold tabular-nums text-ink">
				{fmt(Math.abs(total))}
			</p>
			<p class="font-mono text-[10px] tabular-nums text-ink-faint">
				{priorShortLabel} {priorVerb} {fmt(Math.abs(prior))}
			</p>
		</div>
	</div>
</div>
