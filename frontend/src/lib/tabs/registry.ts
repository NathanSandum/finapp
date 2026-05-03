import OverviewView from '$lib/views/overview/View.svelte';
import IncomeView from '$lib/views/income/View.svelte';
import CashFlowView from '$lib/views/cash-flow/View.svelte';
import BalanceSheetView from '$lib/views/balance-sheet/View.svelte';
import EntriesView from '$lib/views/entries/View.svelte';
import type { Tab } from './types';

export const tabs: Tab[] = [
	{ id: 'overview', label: 'Overview', view: OverviewView },
	{ id: 'income', label: 'Income', view: IncomeView },
	{ id: 'cash-flow', label: 'Cash Flow', view: CashFlowView },
	{ id: 'balance-sheet', label: 'Balance Sheet', view: BalanceSheetView },
	{ id: 'entries', label: 'Entries', view: EntriesView }
];
