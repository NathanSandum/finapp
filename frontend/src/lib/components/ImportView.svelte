<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { Account } from '$lib/types';
	import {
		type ColumnMapping,
		type DateRow,
		parseCSV,
		parseDollars,
		autoDetectMappings as detectMappings,
		findAccountByName,
		findRootByType,
		collectNewAccounts,
		buildOpeningBalanceEntry,
		buildActivityEntries
	} from '$lib/import';
	import { onMount } from 'svelte';

	import ImportUploadStep from './ImportUploadStep.svelte';
	import ImportMappingStep from './ImportMappingStep.svelte';
	import ImportPreviewStep from './ImportPreviewStep.svelte';
	import ImportProgressStep from './ImportProgressStep.svelte';
	import ImportDoneStep from './ImportDoneStep.svelte';

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

	async function loadAccounts() {
		const data = await api.listAccounts(workspaceId);
		accounts = data.accounts;
	}

	onMount(() => {
		loadAccounts().catch(() => {
			/* no accounts yet */
		});
	});

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

	// ── File Upload ──────────────────────────────────────────────────────────

	async function handleFileUpload(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		fileError = '';

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
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Import CSV</h2>
	</header>

	{#if step === 'upload'}
		<ImportUploadStep
			bind:year
			{fileError}
			onfileupload={handleFileUpload}
		/>
	{:else if step === 'mapping'}
		<ImportMappingStep
			bind:mappings
			{accounts}
			{dateRows}
			{startRow}
			{mappingErrors}
			onpreview={() => (step = 'preview')}
			onback={() => (step = 'upload')}
		/>
	{:else if step === 'preview'}
		<ImportPreviewStep
			{mappings}
			{accounts}
			{startRow}
			{dateRows}
			{previewStats}
			onimport={executeImport}
			onback={() => (step = 'mapping')}
		/>
	{:else if step === 'importing'}
		<ImportProgressStep
			{importProgress}
			{importTotal}
			{entriesCreated}
		/>
	{:else if step === 'done'}
		<ImportDoneStep
			{entriesCreated}
			{importErrors}
			onreset={() => (step = 'upload')}
		/>
	{/if}
</div>
