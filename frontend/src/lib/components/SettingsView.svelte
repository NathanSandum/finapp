<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Account } from '$lib/types';
	import ImportView from './ImportView.svelte';

	let { workspaceId }: { workspaceId: string } = $props();

	type Tab = 'Accounts' | 'Import' | 'Workspace';
	let activeTab: Tab = $state('Accounts');
	const tabs: Tab[] = ['Accounts', 'Import', 'Workspace'];

	// ── Accounts tab state ───────────────────────────────────────────────────

	let accounts: Account[] = $state([]);
	let editingAccountId: string | null = $state(null);
	let editingAccountName: string = $state('');
	let accountError: string = $state('');

	type AccountNode = {
		account: Account;
		children: AccountNode[];
	};

	type AccountTypeGroup = {
		type: string;
		root: Account;
		tree: AccountNode[];
	};

	function getDescendantIds(node: AccountNode): string[] {
		const ids: string[] = [node.account.id];
		for (const child of node.children) {
			ids.push(...getDescendantIds(child));
		}
		return ids;
	}

	function buildTree(accounts: Account[], parentId: string): AccountNode[] {
		return accounts
			.filter((account) => account.parent_id === parentId)
			.map((account) => ({
				account,
				children: buildTree(accounts, account.id)
			}));
	}

	let accountTypeGroups = $derived.by(() => {
		const roots = accounts.filter((account) => account.parent_id === null);
		const groups: AccountTypeGroup[] = [];

		for (const root of roots) {
			groups.push({
				type: root.account_type,
				root,
				tree: buildTree(accounts, root.id)
			});
		}

		return groups;
	});

	function validParents(account: Account): Account[] {
		// Find the node for this account to get descendants
		function findNode(nodes: AccountNode[]): AccountNode | null {
			for (const node of nodes) {
				if (node.account.id === account.id) return node;
				const found = findNode(node.children);
				if (found) return found;
			}
			return null;
		}

		const descendantIds = new Set<string>();
		for (const group of accountTypeGroups) {
			for (const node of group.tree) {
				const found = findNode([node]);
				if (found) {
					for (const id of getDescendantIds(found)) {
						descendantIds.add(id);
					}
				}
			}
		}

		return accounts.filter(
			(candidate) =>
				candidate.account_type === account.account_type &&
				candidate.id !== account.id &&
				!descendantIds.has(candidate.id) &&
				candidate.parent_id !== null // don't allow reparenting to root
				|| (candidate.parent_id === null && candidate.account_type === account.account_type && candidate.id !== account.id)
		);
	}

	function isLeaf(account: Account): boolean {
		return !accounts.some((other) => other.parent_id === account.id);
	}

	function isRoot(account: Account): boolean {
		return account.parent_id === null;
	}

	async function loadAccounts() {
		try {
			const data = await api.listAccounts(workspaceId);
			accounts = data.accounts;
			accountError = '';
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			accountError = errorData.error ?? errorData.message ?? 'Failed to load accounts';
		}
	}

	function startEditingAccount(account: Account) {
		editingAccountId = account.id;
		editingAccountName = account.name;
	}

	async function saveAccountName(account: Account) {
		if (editingAccountName.trim() === '' || editingAccountName === account.name) {
			editingAccountId = null;
			return;
		}
		try {
			await api.updateAccount(workspaceId, account.id, { name: editingAccountName.trim() });
			editingAccountId = null;
			await loadAccounts();
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			accountError = errorData.error ?? errorData.message ?? 'Failed to rename account';
		}
	}

	async function reparentAccount(account: Account, newParentId: string) {
		try {
			await api.updateAccount(workspaceId, account.id, { parent_id: newParentId });
			await loadAccounts();
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			accountError = errorData.error ?? errorData.message ?? 'Failed to reparent account';
		}
	}

	async function handleDeleteAccount(account: Account) {
		try {
			await api.deleteAccount(workspaceId, account.id);
			await loadAccounts();
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			accountError = errorData.error ?? errorData.message ?? 'Failed to delete account';
		}
	}

	// ── Workspace tab state ──────────────────────────────────────────────────

	let workspaceName: string = $state('');
	let editingWorkspace: boolean = $state(false);
	let editingWorkspaceName: string = $state('');
	let workspaceError: string = $state('');

	async function loadWorkspace() {
		try {
			const data = await api.listWorkspaces();
			const workspace = data.workspaces.find(
				(workspace: { workspace: string }) => workspace.workspace === workspaceId
			);
			if (workspace) {
				workspaceName = workspace.name;
			}
			workspaceError = '';
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			workspaceError = errorData.error ?? errorData.message ?? 'Failed to load workspace';
		}
	}

	function startEditingWorkspace() {
		editingWorkspace = true;
		editingWorkspaceName = workspaceName;
	}

	async function saveWorkspaceName() {
		if (editingWorkspaceName.trim() === '' || editingWorkspaceName === workspaceName) {
			editingWorkspace = false;
			return;
		}
		try {
			await api.updateWorkspace(workspaceId, { name: editingWorkspaceName.trim() });
			workspaceName = editingWorkspaceName.trim();
			editingWorkspace = false;
			workspaceError = '';
		} catch (error: unknown) {
			const errorData = error as { error?: string; message?: string };
			workspaceError = errorData.error ?? errorData.message ?? 'Failed to rename workspace';
		}
	}

	// ── Load data on mount / tab change ──────────────────────────────────────

	$effect(() => {
		if (activeTab === 'Accounts') {
			loadAccounts();
		} else if (activeTab === 'Workspace') {
			loadWorkspace();
		}
	});
</script>

{#snippet accountRow(node: AccountNode, depth: number)}
	{@const account = node.account}
	<div
		class="flex items-center gap-2 border-b border-gray-800 py-1.5"
		style="padding-left: {depth * 1.5}rem"
	>
		{#if editingAccountId === account.id}
			<input
				class="flex-1 rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
				value={editingAccountName}
				oninput={(event) => (editingAccountName = (event.target as HTMLInputElement).value)}
				onkeydown={(event) => {
					if (event.key === 'Enter') saveAccountName(account);
					if (event.key === 'Escape') (editingAccountId = null);
				}}
				onblur={() => saveAccountName(account)}
			/>
		{:else}
			<button
				class="flex-1 text-left text-sm text-gray-200 hover:text-white"
				onclick={() => startEditingAccount(account)}
			>
				{account.name}
			</button>
		{/if}

		{#if !isRoot(account)}
			<select
				class="rounded border border-gray-700 bg-gray-800 px-1 py-0.5 text-xs text-gray-400"
				value={account.parent_id}
				onchange={(event) => reparentAccount(account, (event.target as HTMLSelectElement).value)}
			>
				{#each validParents(account) as parent}
					<option value={parent.id} selected={parent.id === account.parent_id}>
						{parent.name}
					</option>
				{/each}
			</select>
		{/if}

		{#if !isRoot(account) && isLeaf(account)}
			<button
				class="text-sm text-gray-500 hover:text-red-400"
				onclick={() => handleDeleteAccount(account)}
				title="Delete account"
			>
				x
			</button>
		{/if}
	</div>

	{#each node.children as child}
		{@render accountRow(child, depth + 1)}
	{/each}
{/snippet}

<div class="flex h-full flex-col">
	<!-- Tab bar -->
	<div class="flex border-b border-gray-800">
		{#each tabs as tab}
			<button
				class="px-4 py-2 text-sm {activeTab === tab
					? 'border-b-2 border-blue-500 text-white'
					: 'text-gray-400 hover:text-gray-200'}"
				onclick={() => (activeTab = tab)}
			>
				{tab}
			</button>
		{/each}
	</div>

	<!-- Content -->
	<div class="flex-1 overflow-auto">
		{#if activeTab === 'Accounts'}
			<div class="p-4">
				{#if accountError}
					<div class="mb-4 rounded border border-red-900 bg-red-950/50 px-3 py-2 text-sm text-red-400">
						{accountError}
					</div>
				{/if}

				{#each accountTypeGroups as group}
					<div class="mb-6">
						<h3 class="mb-2 text-xs font-semibold uppercase tracking-wider text-gray-500">
							{group.type}
						</h3>
						<div class="text-sm text-gray-300">
							<div class="flex items-center border-b border-gray-800 py-1.5 font-medium text-gray-400">
								{group.root.name}
							</div>
							{#each group.tree as node}
								{@render accountRow(node, 1)}
							{/each}
						</div>
					</div>
				{/each}

				{#if accounts.length === 0 && !accountError}
					<p class="text-sm text-gray-500">No accounts found.</p>
				{/if}
			</div>
		{:else if activeTab === 'Import'}
			<ImportView {workspaceId} />
		{:else if activeTab === 'Workspace'}
			<div class="p-4">
				{#if workspaceError}
					<div class="mb-4 rounded border border-red-900 bg-red-950/50 px-3 py-2 text-sm text-red-400">
						{workspaceError}
					</div>
				{/if}

				<div class="mb-4">
					<span class="mb-1 block text-xs font-semibold uppercase tracking-wider text-gray-500">
						Workspace Name
					</span>
					{#if editingWorkspace}
						<input
							class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
							value={editingWorkspaceName}
							oninput={(event) => (editingWorkspaceName = (event.target as HTMLInputElement).value)}
							onkeydown={(event) => {
								if (event.key === 'Enter') saveWorkspaceName();
								if (event.key === 'Escape') (editingWorkspace = false);
							}}
							onblur={() => saveWorkspaceName()}
						/>
					{:else}
						<button
							class="text-sm text-gray-200 hover:text-white"
							onclick={() => startEditingWorkspace()}
						>
							{workspaceName || '...'}
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</div>
</div>
