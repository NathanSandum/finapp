<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { fmt } from '$lib/format';
	import Num from '$lib/components/Num.svelte';
	import { deleteRecurring } from './api';
	import { scheduleLabel } from './schedules';
	import type { RecurringRule } from './loader';

	let {
		rules,
		accountNames,
		emptyMessage = 'No recurring rules yet.'
	}: {
		rules: RecurringRule[];
		accountNames: Record<string, string>;
		emptyMessage?: string;
	} = $props();

	let busyId = $state<string | null>(null);
	let errorMessage = $state<string | null>(null);

	function templateAmount(rule: RecurringRule): number {
		return rule.templates
			.filter((t) => t.amount > 0)
			.reduce((sum, t) => sum + t.amount, 0);
	}

	function counterpartyLabel(rule: RecurringRule): string {
		const debit = rule.templates.find((t) => t.amount > 0);
		const credit = rule.templates.find((t) => t.amount < 0);
		const debitName = debit ? accountNames[debit.account] ?? '?' : '?';
		const creditName = credit ? accountNames[credit.account] ?? '?' : '?';
		return `${creditName} → ${debitName}`;
	}

	async function commitDelete(rule: RecurringRule) {
		if (!confirm(`Delete recurring rule "${rule.description}"?`)) return;
		busyId = rule.id;
		errorMessage = null;
		const result = await deleteRecurring(rule.id);
		busyId = null;
		if (result.status === 'success') {
			cache.invalidateAll();
		} else if (result.status === 'fail') {
			errorMessage = String(Object.values(result.data)[0] ?? 'delete failed');
		} else {
			errorMessage = result.message;
		}
	}
</script>

{#if rules.length === 0}
	<p class="border border-dashed border-border-faint px-3 py-4 text-[12px] text-ink-muted">
		{emptyMessage}
	</p>
{:else}
	<ul class="divide-y divide-border-faint border border-border bg-surface-card">
		{#each rules as rule (rule.id)}
			<li class="flex items-center gap-3 px-3 py-2 text-sm">
				<div class="flex-1 min-w-0">
					<div class="flex items-baseline gap-2">
						<span class="font-medium text-ink truncate">{rule.description}</span>
						<span class="font-mono text-[10px] uppercase tracking-[0.06em] text-ink-muted">
							{scheduleLabel(rule.rrule)}
						</span>
					</div>
					<div class="text-[11px] text-ink-muted truncate">
						{counterpartyLabel(rule)}
						{#if rule.next_pending}
							· next {rule.next_pending}
						{/if}
					</div>
				</div>
				<span class="font-mono text-sm tabular-nums text-ink">
					<Num text={fmt(templateAmount(rule))} />
				</span>
				<button
					type="button"
					onclick={() => commitDelete(rule)}
					disabled={busyId === rule.id}
					class="border border-border bg-surface-card px-2 py-1 text-[11px] text-ink-muted hover:bg-surface-hover-faint disabled:opacity-50"
				>
					{busyId === rule.id ? '…' : 'delete'}
				</button>
			</li>
		{/each}
	</ul>
{/if}

{#if errorMessage}
	<p class="mt-2 text-[11px] text-negative">{errorMessage}</p>
{/if}
