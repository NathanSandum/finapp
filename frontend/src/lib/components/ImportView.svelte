<script lang="ts">
	import * as api from '$lib/api.svelte';
	import {
		type Account,
		type MappingType,
		type ColumnMapping,
		type DateRow,
		parseCSV,
		parseDollars,
		formatAmount,
		autoDetectMappings as detectMappings,
		findAccountByName,
		findRootByType,
		collectNewAccounts,
		buildOpeningBalanceEntry,
		buildActivityEntries
	} from '$lib/import';
	import { onMount } from 'svelte';

	const MAPPING_LABELS: Record<MappingType, string> = {
		skip: 'Skip',
		opening_balance: 'Opening Balance',
		income: 'Income',
		expense: 'Expense',
		deduction: 'Deduction',
		transfer: 'Transfer',
		credit_expense: 'Credit Expense'
	};

	let { workspaceId }: { workspaceId: string } = $props();

	let accounts: Account[] = $state([]);
	let year = $state(new Date().getFullYear());

	// CSV state
	let headers: string[] = $state([]);
	let startRow: string[] = $state([]);
	let dateRows: DateRow[] = $state([]);
	let mappings: ColumnMapping[] = $state([]);

	// UI state
	type Step = 'upload' | 'mapping' | 'preview' | 'importing' | 'done';
	let step: Step = $state('upload');
	let importProgress = $state(0);
	let importTotal = $state(0);
	let importErrors: string[] = $state([]);
	let entriesCreated = $state(0);
	let fileError = $state('');

	let accountsById = $derived(Object.fromEntries(accounts.map((a) => [a.id, a])));

	let leafAccounts = $derived(accounts.filter((a) => a.parent_id !== null));

	async function loadAccounts() {
		const data = await api.listAccounts(workspaceId);
		accounts = data.accounts;
	}

	onMount(() => {
		loadAccounts().catch(() => {
			/* no accounts yet */
		});
	});

	// ── File Upload ──────────────────────────────────────────────────────────

	async function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		fileError = '';

		// Ensure accounts are loaded before parsing
		if (accounts.length === 0) {
			try {
				await loadAccounts();
			} catch (e: unknown) {
				const err = e as { error?: string; message?: string };
				fileError = `Failed to load accounts (workspace=${workspaceId}): ${err.message ?? err.error ?? JSON.stringify(e)}`;
				return;
			}
		}

		const reader = new FileReader();
		reader.onload = (e) => {
			try {
				const text = e.target?.result as string;
				const parsed = parseCSV(text, year);
				headers = parsed.headers;
				startRow = parsed.startRow;
				dateRows = parsed.dateRows;
				mappings = detectMappings(headers, accounts);
				step = 'mapping';
			} catch (err: unknown) {
				fileError = (err as Error).message;
			}
		};
		reader.readAsText(file);
	}

	// ── Validation ───────────────────────────────────────────────────────────

	let mappingErrors = $derived.by(() => {
		const errors: string[] = [];
		for (const mapping of mappings) {
			if (mapping.type === 'transfer' && !mapping.accountId) {
				errors.push(`"${mapping.column}": Transfer requires selecting an existing destination account`);
			}
			if (mapping.type !== 'skip' && mapping.type !== 'opening_balance' && !mapping.accountId && !mapping.newAccountName) {
				errors.push(`"${mapping.column}": No account selected`);
			}
		}
		return errors;
	});

	// ── Preview Stats ────────────────────────────────────────────────────────

	let previewStats = $derived.by(() => {
		let openingCount = 0;
		let activityCount = 0;
		let newAccountCount = 0;

		for (let i = 0; i < mappings.length; i++) {
			const mapping = mappings[i];
			if (!mapping.accountId && mapping.newAccountName && mapping.type !== 'skip') {
				newAccountCount++;
			}
			if (mapping.type === 'opening_balance') {
				const cents = parseDollars(startRow[i + 1] ?? '');
				if (cents > 0) openingCount++;
			} else if (mapping.type !== 'skip') {
				for (const row of dateRows) {
					const cents = parseDollars(row.cells[i + 1] ?? '');
					if (cents > 0) activityCount++;
				}
			}
		}

		return { openingCount, activityCount, newAccountCount };
	});

	let sampleEntries = $derived.by(() => {
		const entries: { date: string; column: string; amount: number }[] = [];
		for (const row of dateRows) {
			for (let i = 0; i < mappings.length; i++) {
				const mapping = mappings[i];
				if (mapping.type === 'skip' || mapping.type === 'opening_balance') continue;
				const cents = parseDollars(row.cells[i + 1] ?? '');
				if (cents === 0) continue;
				entries.push({ date: row.date, column: mapping.column, amount: cents });
				if (entries.length >= 20) return entries;
			}
		}
		return entries;
	});

	// ── Import Execution ─────────────────────────────────────────────────────

	async function executeImport() {
		step = 'importing';
		importErrors = [];
		entriesCreated = 0;
		importProgress = 0;

		// 1. Create new accounts
		const createdAccountIds = new Map<string, string>();
		const newAccounts = collectNewAccounts(mappings, startRow, dateRows);

		for (const [name, info] of newAccounts) {
			const root = findRootByType(accounts, info.parentType);
			if (!root) {
				importErrors.push(`No ${info.parentType} root for "${name}"`);
				continue;
			}
			try {
				const result = await api.createAccount(workspaceId, name, { parentId: root.id });
				createdAccountIds.set(name, result.id);
			} catch (e: unknown) {
				const err = e as { error?: string };
				importErrors.push(`Create "${name}" failed: ${err.error ?? 'unknown'}`);
			}
		}

		// Refresh accounts to pick up new ones
		try {
			const data = await api.listAccounts(workspaceId);
			accounts = data.accounts;
		} catch {
			/* continue with what we have */
		}

		function resolveAccountId(mapping: ColumnMapping): string {
			if (mapping.accountId) return mapping.accountId;
			return createdAccountIds.get(mapping.newAccountName) ?? '';
		}

		// 2. Find or create Opening Balances account
		let openingBalancesId = findAccountByName(accounts, 'Opening Balances')?.id ?? '';
		if (!openingBalancesId) {
			const equityRoot = findRootByType(accounts, 'equity');
			if (equityRoot) {
				try {
					const result = await api.createAccount(workspaceId, 'Opening Balances', {
						parentId: equityRoot.id
					});
					openingBalancesId = result.id;
				} catch {
					importErrors.push('Failed to create Opening Balances account');
				}
			}
		}

		// 3. Create opening balance entry
		if (startRow.length > 0 && openingBalancesId) {
			const entry = buildOpeningBalanceEntry(
				mappings,
				startRow,
				dateRows,
				year,
				openingBalancesId,
				accounts
			);
			if (entry) {
				try {
					await api.createEntry(
						workspaceId,
						entry.date,
						entry.description,
						entry.postings,
						true
					);
					entriesCreated++;
				} catch (e: unknown) {
					const err = e as { error?: string };
					importErrors.push(
						`Opening balance entry failed: ${err.error ?? 'unknown'}`
					);
				}
			}
			importProgress++;
		}

		// 4. Create activity entries
		const activityEntries = buildActivityEntries(mappings, dateRows, resolveAccountId);
		importTotal = activityEntries.length + (startRow.length > 0 ? 1 : 0);

		for (const entry of activityEntries) {
			try {
				await api.createEntry(
					workspaceId,
					entry.date,
					entry.description,
					entry.postings,
					true
				);
				entriesCreated++;
			} catch (e: unknown) {
				const err = e as { error?: string };
				importErrors.push(
					`${entry.date} ${entry.description}: ${err.error ?? 'unknown'}`
				);
			}
			importProgress++;
		}

		step = 'done';
	}

	// ── UI Helpers ───────────────────────────────────────────────────────────

	function updateMappingType(index: number, newType: MappingType) {
		const mapping = mappings[index];
		mapping.type = newType;

		const chequingId = findAccountByName(accounts, 'Chequing')?.id ?? '';
		const creditCardId = findAccountByName(accounts, 'Credit Card')?.id ?? '';

		if (newType === 'credit_expense') {
			mapping.fundingAccountId = creditCardId;
		} else if (['income', 'expense', 'deduction', 'transfer'].includes(newType)) {
			mapping.fundingAccountId = chequingId;
		} else {
			mapping.fundingAccountId = '';
		}

		// For transfer, clear "create new" — user must pick an existing account
		if (newType === 'transfer') {
			if (!mapping.accountId) {
				mapping.newAccountName = '';
			}
		}
	}

	function updateMappingAccount(index: number, value: string) {
		if (value === '__new__') {
			mappings[index].accountId = '';
			if (!mappings[index].newAccountName) {
				mappings[index].newAccountName = mappings[index].column;
			}
		} else {
			mappings[index].accountId = value;
			mappings[index].newAccountName = '';
		}
	}

	function needsFunding(type: MappingType): boolean {
		return ['income', 'expense', 'deduction', 'transfer', 'credit_expense'].includes(type);
	}
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Import CSV</h2>
	</header>

	{#if step === 'upload'}
		<div class="mx-auto max-w-lg space-y-4">
			<div>
				<label class="mb-1 block text-sm text-gray-400" for="year-input">Year</label>
				<input
					id="year-input"
					type="number"
					bind:value={year}
					class="w-24 rounded border border-gray-700 bg-gray-900 px-3 py-2 text-sm text-gray-200"
				/>
			</div>
			<div>
				<label class="mb-1 block text-sm text-gray-400" for="csv-input">CSV File</label>
				<input
					id="csv-input"
					type="file"
					accept=".csv"
					onchange={handleFileUpload}
					class="text-sm text-gray-400 file:mr-3 file:rounded file:border-0 file:bg-gray-800 file:px-3 file:py-2 file:text-sm file:text-gray-300 hover:file:bg-gray-700"
				/>
			</div>
			{#if fileError}
				<p class="text-sm text-red-400">{fileError}</p>
			{/if}
		</div>
	{:else if step === 'mapping'}
		<div class="mb-4 text-sm text-gray-500">
			{dateRows.length} days parsed{startRow.length > 0 ? ', Start row found' : ''} &middot; {accounts.length} accounts loaded
		</div>

		<div class="space-y-2">
			{#each mappings as mapping, index}
				<div class="rounded-lg border border-gray-800 bg-gray-900/50 px-4 py-3">
					<div class="flex items-center gap-3">
						<span class="w-48 shrink-0 text-sm font-medium text-gray-200">{mapping.column}</span>
						<select
							class="rounded border border-gray-700 bg-gray-900 px-2 py-1.5 text-sm text-gray-300"
							value={mapping.type}
							onchange={(e) =>
								updateMappingType(
									index,
									(e.target as HTMLSelectElement).value as MappingType
								)}
						>
							{#each Object.entries(MAPPING_LABELS) as [value, label]}
								<option {value}>{label}</option>
							{/each}
						</select>
					</div>

					{#if mapping.type === 'opening_balance'}
						<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
							<span>Starting balance of</span>
							{@render accountSelect(index)}
						</div>
					{:else if mapping.type === 'income'}
						<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
							<span>Revenue from</span>
							{@render accountSelect(index)}
							<span>deposited into</span>
							{@render fundingSelect(index)}
						</div>
					{:else if mapping.type === 'expense' || mapping.type === 'deduction'}
						<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
							<span>Expense categorized as</span>
							{@render accountSelect(index)}
							<span>paid from</span>
							{@render fundingSelect(index)}
						</div>
					{:else if mapping.type === 'credit_expense'}
						<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
							<span>Expense categorized as</span>
							{@render accountSelect(index)}
							<span>charged to</span>
							{@render fundingSelect(index)}
						</div>
					{:else if mapping.type === 'transfer'}
						<div class="mt-2 flex items-center gap-2 text-sm text-gray-400">
							<span>Move money from</span>
							{@render fundingSelect(index)}
							<span>to</span>
							{@render accountSelect(index)}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		{#snippet accountSelect(index: number)}
			<div class="inline-flex flex-col gap-1">
				<select
					class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
					value={mappings[index].accountId || '__new__'}
					onchange={(e) =>
						updateMappingAccount(index, (e.target as HTMLSelectElement).value)}
				>
					<option value="__new__">+ Create new</option>
					{#each leafAccounts as account}
						<option value={account.id}>{account.name}</option>
					{/each}
				</select>
				{#if !mappings[index].accountId}
					<input
						type="text"
						bind:value={mappings[index].newAccountName}
						placeholder="Account name"
						class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
					/>
				{/if}
			</div>
		{/snippet}

		{#snippet fundingSelect(index: number)}
			<select
				class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-200"
				bind:value={mappings[index].fundingAccountId}
			>
				{#each leafAccounts as account}
					<option value={account.id}>{account.name}</option>
				{/each}
			</select>
		{/snippet}

		{#if mappingErrors.length > 0}
			<div class="mt-4 rounded-lg border border-red-900 bg-red-950/50 px-4 py-3">
				{#each mappingErrors as error}
					<div class="text-sm text-red-400">{error}</div>
				{/each}
			</div>
		{/if}

		<div class="mt-4 flex items-center gap-4">
			<button
				class="rounded bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-40 disabled:cursor-not-allowed"
				onclick={() => (step = 'preview')}
				disabled={mappingErrors.length > 0}
			>
				Preview
			</button>
			<button
				class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
				onclick={() => (step = 'upload')}
			>
				Back
			</button>
		</div>
	{:else if step === 'preview'}
		<div class="mb-6 grid grid-cols-3 gap-4">
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Opening Balances</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-200"
					>{previewStats.openingCount} postings</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>Activity Entries</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-200"
					>{previewStats.activityCount}</span
				>
			</div>
			<div class="rounded-lg border border-gray-800 bg-gray-900 px-4 py-3">
				<span class="block text-[11px] font-semibold uppercase tracking-wide text-gray-600"
					>New Accounts</span
				>
				<span class="text-xl font-semibold tabular-nums text-gray-200"
					>{previewStats.newAccountCount}</span
				>
			</div>
		</div>

		{#if startRow.length > 0}
			<div class="mb-4">
				<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
					Opening Balances
				</h3>
				<div class="space-y-1">
					{#each mappings as mapping, index}
						{#if mapping.type === 'opening_balance'}
							{@const cents = parseDollars(startRow[index + 1] ?? '')}
							{#if cents > 0}
								<div class="flex items-center justify-between rounded border border-gray-800 px-3 py-1.5 text-sm">
									<span class="text-gray-300"
										>{mapping.accountId
											? (accountsById[mapping.accountId]?.name ?? mapping.column)
											: mapping.newAccountName}</span
									>
									<span class="font-medium tabular-nums text-emerald-400"
										>{formatAmount(cents)}</span
									>
								</div>
							{/if}
						{/if}
					{/each}
				</div>
			</div>
		{/if}

		<div class="mb-4">
			<h3 class="mb-2 text-[11px] font-semibold uppercase tracking-wide text-gray-600">
				Sample Activity (first 20)
			</h3>
			<div class="space-y-1">
				{#each sampleEntries as entry}
					<div class="flex items-center justify-between rounded border border-gray-800 px-3 py-1.5 text-sm">
						<div class="flex items-center gap-3">
							<span class="tabular-nums text-gray-500">{entry.date}</span>
							<span class="text-gray-300">{entry.column}</span>
						</div>
						<span class="font-medium tabular-nums text-gray-200"
							>{formatAmount(entry.amount)}</span
						>
					</div>
				{/each}
			</div>
		</div>

		<div class="mt-4 flex items-center gap-4">
			<button
				class="rounded bg-emerald-600 px-4 py-2 text-sm font-medium text-white hover:bg-emerald-500"
				onclick={executeImport}
			>
				Import {previewStats.activityCount + (previewStats.openingCount > 0 ? 1 : 0)} entries
			</button>
			<button
				class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
				onclick={() => (step = 'mapping')}
			>
				Back
			</button>
		</div>
	{:else if step === 'importing'}
		<div class="mx-auto max-w-lg space-y-4 py-12">
			<div class="text-center text-sm text-gray-400">
				Importing... {importProgress} / {importTotal}
			</div>
			<div class="h-2 overflow-hidden rounded-full bg-gray-800">
				<div
					class="h-full rounded-full bg-blue-500 transition-all"
					style="width: {importTotal > 0 ? (importProgress / importTotal) * 100 : 0}%"
				></div>
			</div>
			<div class="text-center text-sm text-gray-500">{entriesCreated} entries created</div>
		</div>
	{:else if step === 'done'}
		<div class="mx-auto max-w-lg space-y-4 py-12">
			<div class="rounded-lg border border-emerald-900 bg-emerald-950/50 px-4 py-3 text-center">
				<span class="text-lg font-semibold text-emerald-400">{entriesCreated}</span>
				<span class="text-sm text-emerald-400"> entries created</span>
			</div>

			{#if importErrors.length > 0}
				<div class="rounded-lg border border-red-900 bg-red-950/50 px-4 py-3">
					<h3 class="mb-2 text-sm font-medium text-red-400">
						{importErrors.length} errors
					</h3>
					<div class="max-h-48 space-y-1 overflow-y-auto text-xs text-red-400/80">
						{#each importErrors as error}
							<div>{error}</div>
						{/each}
					</div>
				</div>
			{/if}

			<div class="text-center">
				<button
					class="rounded border border-gray-700 px-4 py-2 text-sm text-gray-400 hover:bg-gray-800"
					onclick={() => (step = 'upload')}
				>
					Import another
				</button>
			</div>
		</div>
	{/if}
</div>
