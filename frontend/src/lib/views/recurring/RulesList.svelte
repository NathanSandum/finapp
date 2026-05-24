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
	<p class="border border-dashed border-border-faint px-4 py-6 text-center text-[12px] text-ink-muted">
		{emptyMessage}
	</p>
{:else}
	<ul class="divide-y divide-border-faint border border-border bg-surface-card">
		{#each rules as rule (rule.id)}
			{@const isConfirming = confirmingId === rule.id}
			<li
				animate:flip={{ duration: DURATION.base }}
				in:rowEnter
				out:rowExit
				class="group flex items-center gap-3 px-4 py-3 text-[13px] transition-colors hover:bg-surface-hover-faint"
			>
				<div class="min-w-0 flex-1">
					<div class="flex items-baseline gap-2">
						<span class="truncate font-medium text-ink">{rule.description}</span>
						<span class="font-ui text-[10px] uppercase tracking-[0.08em] text-ink-muted">
							{scheduleLabel(rule.schedule)}
						</span>
					</div>
					<div class="truncate text-[11px] text-ink-muted">
						{counterpartyLabel(rule)}
						{#if rule.next_pending}
							<span class="opacity-50">·</span> next {rule.next_pending}
						{/if}
					</div>
				</div>
				<span class="font-mono text-[14px] tabular-nums text-ink">
					<Num text={fmt(templateAmount(rule))} />
				</span>

				{#if isConfirming}
					<button
						type="button"
						onclick={() => (confirmingId = null)}
						disabled={busyId === rule.id}
						class="border border-border bg-surface-card px-2 py-1 text-[10px] uppercase tracking-[0.06em] text-ink-body hover:border-ink-faint disabled:opacity-50"
					>
						cancel
					</button>
					<button
						type="button"
						onclick={() => commitDelete(rule)}
						disabled={busyId === rule.id}
						class="border border-negative-border bg-negative-tint px-2 py-1 text-[10px] uppercase tracking-[0.06em] text-negative-strong hover:border-negative-strong disabled:opacity-50"
					>
						{busyId === rule.id ? '…' : 'confirm'}
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (confirmingId = rule.id)}
						class="border border-border bg-surface-card px-2 py-1 text-[10px] uppercase tracking-[0.06em] text-ink-body opacity-0 transition-opacity hover:border-negative-strong hover:text-negative-strong group-hover:opacity-100"
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
