<script lang="ts">
	import { flip } from 'svelte/animate';
	import { fly } from 'svelte/transition';
	import { recurring } from '$lib/data';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import { scheduleLabel } from './schedules';
	import type { RecurringRule } from '$lib/data';
	import { DURATION, rowEnter, rowExit } from '$lib/utils/motion';

	let {
		rules,
		accountNames,
		emptyMessage = 'No recurring rules yet.'
	}: {
		rules: RecurringRule[];
		accountNames: Record<string, string>;
		emptyMessage?: string;
	} = $props();

	let confirmingId = $state<string | null>(null);
	let busyId = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);

	function templateAmount(rule: RecurringRule): number {
		return rule.templates.filter((t) => t.amount > 0).reduce((sum, t) => sum + t.amount, 0);
	}

	function counterpartyLabel(rule: RecurringRule): string {
		const debit = rule.templates.find((t) => t.amount > 0);
		const credit = rule.templates.find((t) => t.amount < 0);
		const debitName = debit ? accountNames[debit.account] ?? '?' : '?';
		const creditName = credit ? accountNames[credit.account] ?? '?' : '?';
		return `${creditName} → ${debitName}`;
	}

	async function commitDelete(rule: RecurringRule) {
		busyId = rule.id;
		errorMessage = null;
		const result = await recurring.remove(rule.id);
		busyId = null;
		confirmingId = null;
		if (result.status === 'fail') {
			errorMessage = String(Object.values(result.data)[0] ?? 'delete failed');
		} else if (result.status === 'error') {
			errorMessage = result.message;
		}
	}
</script>

{#if rules.length === 0}
	<p class="glass-faint rounded-xl px-4 py-6 text-center text-[12px] text-ink-muted">
		{emptyMessage}
	</p>
{:else}
	<ul class="flex flex-col gap-2">
		{#each rules as rule (rule.id)}
			{@const isConfirming = confirmingId === rule.id}
			<li
				animate:flip={{ duration: DURATION.base }}
				in:rowEnter
				out:rowExit
				class="rule-row group relative flex items-center gap-3 rounded-xl px-4 py-3 text-[13px] transition-all duration-300"
			>
				<span class="rule-row__bg pointer-events-none absolute inset-0 rounded-xl"></span>
				<div class="relative min-w-0 flex-1">
					<div class="flex items-baseline gap-2">
						<span class="truncate font-medium text-ink">{rule.description}</span>
						<span class="font-ui text-[10px] uppercase tracking-[0.1em] text-ink-muted">
							{scheduleLabel(rule.schedule)}
						</span>
					</div>
					<div class="truncate text-[11px] text-ink-muted">
						{counterpartyLabel(rule)}
						{#if rule.next_pending}
							<span class="opacity-40">·</span> next {rule.next_pending}
						{/if}
					</div>
				</div>
				<span class="relative font-mono text-[15px] tabular-nums text-ink">
					<Num text={fmt(templateAmount(rule))} />
				</span>

				{#if isConfirming}
					<button
						type="button"
						onclick={() => (confirmingId = null)}
						disabled={busyId === rule.id}
						class="relative rounded-md border border-[oklch(72%_0.012_260_/_0.4)] bg-[oklch(100%_0_0_/_0.7)] px-2 py-1 text-[10px] uppercase tracking-[0.08em] text-ink-body hover:border-ink-faint disabled:opacity-50"
					>
						cancel
					</button>
					<button
						type="button"
						onclick={() => commitDelete(rule)}
						disabled={busyId === rule.id}
						class="relative rounded-md border border-negative-border bg-negative-tint px-2 py-1 text-[10px] uppercase tracking-[0.08em] text-negative-strong hover:border-negative-strong disabled:opacity-50"
					>
						{busyId === rule.id ? '…' : 'confirm'}
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (confirmingId = rule.id)}
						class="relative rounded-md border border-[oklch(72%_0.012_260_/_0.4)] bg-[oklch(100%_0_0_/_0.5)] px-2 py-1 text-[10px] uppercase tracking-[0.08em] text-ink-body opacity-0 transition-opacity hover:border-negative-strong hover:text-negative-strong group-hover:opacity-100"
					>
						delete
					</button>
				{/if}
			</li>
		{/each}
	</ul>
{/if}

{#if errorMessage}
	<p in:fly={{ y: -3, duration: DURATION.fast }} class="mt-2 text-[11px] text-negative">
		{errorMessage}
	</p>
{/if}

<style>
	.rule-row__bg {
		background: oklch(100% 0 0 / 0.5);
		backdrop-filter: blur(10px);
		-webkit-backdrop-filter: blur(10px);
		box-shadow:
			inset 0 0 0 1px oklch(100% 0 0 / 0.55),
			0 1px 2px oklch(40% 0.05 260 / 0.05);
		transition: all 240ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.rule-row:hover .rule-row__bg {
		background: linear-gradient(
			135deg,
			oklch(75% 0.14 268 / 0.08) 0%,
			oklch(100% 0 0 / 0.7) 70%
		);
		box-shadow:
			inset 0 0 0 1px oklch(58% 0.2 268 / 0.25),
			0 6px 18px -6px oklch(58% 0.2 268 / 0.18);
	}
</style>
