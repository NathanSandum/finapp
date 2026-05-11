<script lang="ts">
	import { cache } from '$lib/api/cache.svelte';
	import { createAccount, deleteAccount, renameAccount } from './api';
	import { ROLE_BADGE, type AccountNode } from './tree';
	import Self from './AccountRow.svelte';

	let {
		node,
		depth = 0
	}: {
		node: AccountNode;
		depth?: number;
	} = $props();

	const account = $derived(node.account);
	const isAnchor = $derived(account.role !== null);
	const hasChildren = $derived(node.children.length > 0);
	const badge = $derived(account.role ? ROLE_BADGE[account.role] : undefined);
	const canDelete = $derived(!isAnchor && !hasChildren);

	let editing = $state(false);
	let nameInput = $state('');
	let adding = $state(false);
	let newName = $state('');
	let busy = $state(false);
	let error = $state<string | null>(null);
	let nameEl = $state<HTMLInputElement | null>(null);
	let newNameEl = $state<HTMLInputElement | null>(null);

	$effect(() => {
		if (editing && nameEl) {
			nameEl.focus();
			nameEl.select();
		}
	});

	$effect(() => {
		if (adding && newNameEl) newNameEl.focus();
	});

	function startEdit() {
		nameInput = account.name;
		editing = true;
	}

	async function commitRename() {
		const trimmed = nameInput.trim();
		if (!trimmed || trimmed === account.name) {
			editing = false;
			return;
		}
		busy = true;
		error = null;
		const result = await renameAccount(account.id, trimmed);
		busy = false;
		if (result.status === 'success') {
			cache.invalidateAll();
			editing = false;
		} else {
			error = result.status === 'fail' ? Object.values(result.data)[0] ?? 'rename failed' : result.message;
		}
	}

	async function commitAddChild() {
		const trimmed = newName.trim();
		if (!trimmed) {
			adding = false;
			return;
		}
		busy = true;
		error = null;
		const result = await createAccount({
			name: trimmed,
			parent_id: account.id,
			currency: account.currency
		});
		busy = false;
		if (result.status === 'success') {
			cache.invalidateAll();
			adding = false;
			newName = '';
		} else {
			error = result.status === 'fail' ? Object.values(result.data)[0] ?? 'create failed' : result.message;
		}
	}

	async function commitDelete() {
		if (!canDelete) return;
		if (!confirm(`Delete "${account.name}"?`)) return;
		busy = true;
		error = null;
		const result = await deleteAccount(account.id);
		busy = false;
		if (result.status === 'success') {
			cache.invalidateAll();
		} else {
			error = result.status === 'fail' ? Object.values(result.data)[0] ?? 'delete failed' : result.message;
		}
	}
</script>

<div class="flex items-center gap-2 py-1" style="padding-left: {depth * 16}px;">
	{#if editing}
		<input
			type="text"
			bind:value={nameInput}
			bind:this={nameEl}
			onblur={commitRename}
			onkeydown={(e) => {
				if (e.key === 'Enter') commitRename();
				if (e.key === 'Escape') editing = false;
			}}
			disabled={busy}
			class="flex-1 border border-border bg-surface-card px-1.5 py-0.5 text-sm"
		/>
	{:else}
		<button
			type="button"
			onclick={startEdit}
			class="flex-1 truncate text-left text-sm text-ink hover:text-ink-body"
		>
			{account.name}
		</button>
	{/if}

	{#if badge}
		<span class="bg-surface-sunken px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-[0.06em] text-ink-muted">
			{badge}
		</span>
	{/if}

	<div class="flex items-center gap-1.5">
		<button
			type="button"
			onclick={() => (adding = true)}
			disabled={busy}
			class="inline-flex items-center gap-1 border border-border bg-surface-card px-2 py-1 text-[11px] font-medium uppercase tracking-[0.06em] text-ink-body transition-colors hover:border-indigo-500 hover:text-indigo-500 disabled:opacity-50"
		>
			<span class="text-sm leading-none">+</span>
			<span>new</span>
		</button>
		{#if canDelete}
			<button
				type="button"
				onclick={commitDelete}
				disabled={busy}
				class="inline-flex items-center gap-1 border border-border bg-surface-card px-2 py-1 text-[11px] font-medium uppercase tracking-[0.06em] text-ink-body transition-colors hover:border-negative-strong hover:text-negative-strong disabled:opacity-50"
			>
				<span class="text-sm leading-none">×</span>
				<span>delete</span>
			</button>
		{/if}
	</div>
</div>

{#if error}
	<p class="ml-4 py-0.5 text-[11px] text-negative" style="padding-left: {depth * 16}px;">{error}</p>
{/if}

{#if adding}
	<div class="flex items-center gap-2 py-1" style="padding-left: {(depth + 1) * 16}px;">
		<input
			type="text"
			bind:value={newName}
			bind:this={newNameEl}
			placeholder="new account name"
			onblur={commitAddChild}
			onkeydown={(e) => {
				if (e.key === 'Enter') commitAddChild();
				if (e.key === 'Escape') {
					adding = false;
					newName = '';
				}
			}}
			disabled={busy}
			class="flex-1 border border-border bg-surface-card px-1.5 py-0.5 text-sm"
		/>
	</div>
{/if}

{#each node.children as child (child.account.id)}
	<Self node={child} depth={depth + 1} />
{/each}
