<script lang="ts">
	import * as api from '$lib/api.svelte';
	import type { ImportTransaction } from '$lib/api.svelte';

	let { workspaceId }: { workspaceId: string } = $props();

	// Column type mapping for the daily ledger CSV format
	type ColumnType = 'skip' | 'income' | 'expense' | 'deduction' | 'transfer_savings' | 'credit_payment' | 'credit_expense';

	const DEFAULT_COLUMN_MAP: Record<string, ColumnType> = {
		'Day': 'skip',
		'Chequing Balance': 'skip',
		'Savings Balance': 'skip',
		'Credit Balance': 'skip',
		'Savings Contributions': 'transfer_savings',
		'Employment Income': 'income',
		'Benefits and Tax Credits': 'income',
		'Income Tax': 'deduction',
		'CPP': 'deduction',
		'EI': 'deduction',
		'Investment Income': 'income',
		'Financing Income': 'income',
		'Rent': 'expense',
		'Household Expenses': 'expense',
		'Eating Out': 'expense',
		'Cafes': 'expense',
		'Books': 'expense',
		'Workspace Supplies': 'expense',
		'Other': 'expense',
		'Credit Payment': 'credit_payment',
		'Transit': 'expense',
		'Haircut': 'expense',
		'Banking Fees': 'expense',
		'Recreation on Credit': 'credit_expense',
		'Entrepreneurship on Credit': 'credit_expense',
		'Workplace on Credit': 'credit_expense',
		'Subscriptions on Credit': 'credit_expense',
	};

	let headers: string[] = $state([]);
	let columnMap: Record<string, ColumnType> = $state({});
	let parsedRows: string[][] = $state([]);
	let importing = $state(false);
	let result: { accounts_created: number; categories_created: number; transactions_created: number } | null = $state(null);
	let error = $state('');
	let preview: ImportTransaction[] = $state([]);

	function handleFile(event: Event) {
		const input = event.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		const reader = new FileReader();
		reader.onload = (e) => {
			const text = e.target?.result as string;
			parseCSV(text);
		};
		reader.readAsText(file);
	}

	function parseCSV(text: string) {
		const lines = text.trim().split('\n');
		if (lines.length < 2) return;

		headers = lines[0].split(',').map((h) => h.trim().replace(/^"|"$/g, ''));

		// Auto-detect column types from defaults, fallback to expense
		columnMap = {};
		for (const h of headers) {
			columnMap[h] = DEFAULT_COLUMN_MAP[h] ?? 'expense';
		}

		// Parse data rows (skip header)
		parsedRows = lines.slice(1).map((line) => {
			// Handle quoted CSV fields
			const fields: string[] = [];
			let current = '';
			let inQuotes = false;
			for (const char of line) {
				if (char === '"') {
					inQuotes = !inQuotes;
				} else if (char === ',' && !inQuotes) {
					fields.push(current.trim());
					current = '';
				} else {
					current += char;
				}
			}
			fields.push(current.trim());
			return fields;
		});

		buildPreview();
	}

	function parseDollar(s: string): number {
		if (!s || s === '') return 0;
		const cleaned = s.replace(/[$,"]/g, '').replace(/\((.+)\)/, '-$1');
		const val = parseFloat(cleaned);
		if (isNaN(val)) return 0;
		return Math.round(val * 100);
	}

	function parseDate(dayStr: string): string | null {
		if (!dayStr || dayStr === 'Start') return null;
		// Format: "Jan 02" or "Apr 15" — need to add year
		const months: Record<string, string> = {
			Jan: '01', Feb: '02', Mar: '03', Apr: '04', May: '05', Jun: '06',
			Jul: '07', Aug: '08', Sep: '09', Oct: '10', Nov: '11', Dec: '12',
		};
		const parts = dayStr.split(' ');
		if (parts.length !== 2) return null;
		const month = months[parts[0]];
		if (!month) return null;
		const day = parts[1].padStart(2, '0');
		// Infer year — assume 2026 for now
		return `2026-${month}-${day}`;
	}

	function buildPreview() {
		const transactions: ImportTransaction[] = [];
		const dateIdx = headers.indexOf('Day');

		for (const row of parsedRows) {
			const dateStr = parseDate(row[dateIdx]);
			if (!dateStr) continue;

			for (let i = 0; i < headers.length; i++) {
				const header = headers[i];
				const type = columnMap[header];
				if (type === 'skip') continue;

				const cents = parseDollar(row[i]);
				if (cents === 0) continue;

				if (type === 'income') {
					transactions.push({
						account: 'Chequing',
						account_type: 'chequing',
						date: dateStr,
						description: header,
						amount: cents,
						category: header,
						cleared: true,
					});
				} else if (type === 'expense') {
					transactions.push({
						account: 'Chequing',
						account_type: 'chequing',
						date: dateStr,
						description: header,
						amount: -Math.abs(cents),
						category: header,
						cleared: true,
					});
				} else if (type === 'deduction') {
					transactions.push({
						account: 'Chequing',
						account_type: 'chequing',
						date: dateStr,
						description: header,
						amount: -Math.abs(cents),
						category: header,
						cleared: true,
					});
				} else if (type === 'transfer_savings') {
					// Positive = contribution to savings (outflow from chequing)
					// Negative = withdrawal from savings (inflow to chequing)
					transactions.push({
						account: 'Savings',
						account_type: 'savings',
						date: dateStr,
						description: 'Transfer',
						amount: cents,
						category: 'Transfers',
						cleared: true,
					});
					transactions.push({
						account: 'Chequing',
						account_type: 'chequing',
						date: dateStr,
						description: 'Transfer',
						amount: -cents,
						category: 'Transfers',
						cleared: true,
					});
				} else if (type === 'credit_payment') {
					// Payment from chequing to credit card
					transactions.push({
						account: 'Chequing',
						account_type: 'chequing',
						date: dateStr,
						description: 'Credit Card Payment',
						amount: -Math.abs(cents),
						category: 'Credit Card Payment',
						cleared: true,
					});
					transactions.push({
						account: 'Credit Card',
						account_type: 'credit',
						date: dateStr,
						description: 'Payment Received',
						amount: Math.abs(cents),
						category: 'Credit Card Payment',
						cleared: true,
					});
				} else if (type === 'credit_expense') {
					// Strip " on Credit" suffix for category name
					const categoryName = header.replace(/ on Credit$/, '');
					transactions.push({
						account: 'Credit Card',
						account_type: 'credit',
						date: dateStr,
						description: categoryName,
						amount: -Math.abs(cents),
						category: categoryName,
						cleared: true,
					});
				}
			}
		}

		preview = transactions;
	}

	async function doImport() {
		importing = true;
		error = '';
		result = null;
		try {
			result = await api.importTransactions(workspaceId, preview);
		} catch (e: unknown) {
			const err = e as { message?: string };
			error = err.message || 'Import failed';
		} finally {
			importing = false;
		}
	}

	function formatAmount(cents: number): string {
		const abs = Math.abs(cents);
		const dollars = Math.floor(abs / 100);
		const remainder = abs % 100;
		const sign = cents < 0 ? '-' : '';
		return `${sign}$${dollars.toLocaleString()}.${String(remainder).padStart(2, '0')}`;
	}
</script>

<div>
	<header class="mb-6 border-b border-gray-800 pb-4">
		<h2 class="text-lg font-semibold text-gray-100">Import Transactions</h2>
		<span class="text-sm text-gray-600">Upload a CSV file to bulk-import transactions</span>
	</header>

	{#if result}
		<div class="mb-6 rounded-lg border border-emerald-800 bg-emerald-950/30 px-4 py-3">
			<p class="text-sm font-medium text-emerald-400">Import complete</p>
			<ul class="mt-1 text-sm text-emerald-300/70">
				<li>{result.accounts_created} accounts created</li>
				<li>{result.categories_created} categories created</li>
				<li>{result.transactions_created} transactions imported</li>
			</ul>
		</div>
	{/if}

	{#if error}
		<div class="mb-6 rounded-lg border border-red-800 bg-red-950/30 px-4 py-3 text-sm text-red-400">
			{error}
		</div>
	{/if}

	<div class="mb-6">
		<label class="mb-2 block text-sm font-medium text-gray-300">CSV File</label>
		<input
			type="file"
			accept=".csv"
			onchange={handleFile}
			class="block w-full text-sm text-gray-400 file:mr-4 file:rounded-lg file:border file:border-gray-700 file:bg-gray-800 file:px-4 file:py-2 file:text-sm file:font-medium file:text-gray-300 hover:file:bg-gray-700"
		/>
	</div>

	{#if headers.length > 0}
		<!-- Column mapping -->
		<div class="mb-6">
			<h3 class="mb-2 text-sm font-medium text-gray-300">Column Mapping</h3>
			<div class="grid grid-cols-2 gap-2">
				{#each headers as header}
					<div class="flex items-center gap-2">
						<span class="w-48 truncate text-sm text-gray-400" title={header}>{header}</span>
						<select
							class="rounded border border-gray-700 bg-gray-800 px-2 py-1 text-sm text-gray-300"
							bind:value={columnMap[header]}
							onchange={buildPreview}
						>
							<option value="skip">Skip</option>
							<option value="income">Income</option>
							<option value="expense">Expense</option>
							<option value="deduction">Deduction</option>
							<option value="transfer_savings">Transfer (Savings)</option>
							<option value="credit_payment">Credit Payment</option>
							<option value="credit_expense">Credit Expense</option>
						</select>
					</div>
				{/each}
			</div>
		</div>

		<!-- Preview -->
		{#if preview.length > 0}
			<div class="mb-4 flex items-center justify-between">
				<h3 class="text-sm font-medium text-gray-300">
					Preview: {preview.length} transactions
				</h3>
				<button
					class="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-500 disabled:opacity-50"
					onclick={doImport}
					disabled={importing}
				>
					{importing ? 'Importing...' : 'Import'}
				</button>
			</div>

			<div class="max-h-96 overflow-y-auto">
				<table class="w-full">
					<thead>
						<tr class="border-b border-gray-800 text-left text-[11px] font-semibold uppercase tracking-wide text-gray-600">
							<th class="px-3 py-2">Date</th>
							<th class="px-3 py-2">Account</th>
							<th class="px-3 py-2">Category</th>
							<th class="px-3 py-2">Description</th>
							<th class="px-3 py-2 text-right">Amount</th>
						</tr>
					</thead>
					<tbody>
						{#each preview.slice(0, 50) as txn}
							<tr class="border-b border-gray-900 hover:bg-gray-900/50">
								<td class="px-3 py-1.5 text-sm tabular-nums text-gray-500">{txn.date}</td>
								<td class="px-3 py-1.5 text-sm text-gray-400">{txn.account}</td>
								<td class="px-3 py-1.5 text-sm text-gray-300">{txn.category}</td>
								<td class="px-3 py-1.5 text-sm text-gray-400">{txn.description}</td>
								<td class="px-3 py-1.5 text-right text-sm font-medium tabular-nums {txn.amount < 0 ? 'text-red-400' : 'text-emerald-400'}">
									{formatAmount(txn.amount)}
								</td>
							</tr>
						{/each}
						{#if preview.length > 50}
							<tr>
								<td colspan="5" class="px-3 py-2 text-center text-sm text-gray-600">
									... and {preview.length - 50} more
								</td>
							</tr>
						{/if}
					</tbody>
				</table>
			</div>
		{:else}
			<p class="text-sm text-gray-600">No non-zero transactions found in CSV</p>
		{/if}
	{/if}
</div>
