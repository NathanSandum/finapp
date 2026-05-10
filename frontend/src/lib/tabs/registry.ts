import OverviewView from '$lib/views/overview/View.svelte';
import IncomeView from '$lib/views/income/View.svelte';
import CashFlowView from '$lib/views/cash-flow/View.svelte';
import BalanceSheetView from '$lib/views/balance-sheet/View.svelte';
import EntriesView from '$lib/views/entries/View.svelte';
import { load as loadEntries } from '$lib/views/entries/loader';
import { load as loadIncome } from '$lib/views/income/loader';
import { load as loadCashFlow } from '$lib/views/cash-flow/loader';
import { load as loadBalanceSheet } from '$lib/views/balance-sheet/loader';
import type { JSend, Tab } from './types';

const stubLoader = (): Promise<JSend<null>> =>
	Promise.resolve({ status: 'success', data: null });

export const tabs: Tab[] = [
	{ id: 'overview', label: 'Overview', view: OverviewView, loader: stubLoader },
	{ id: 'income', label: 'Income', view: IncomeView, loader: loadIncome },
	{ id: 'cash-flow', label: 'Cash Flow', view: CashFlowView, loader: loadCashFlow },
	{ id: 'balance-sheet', label: 'Balance Sheet', view: BalanceSheetView, loader: loadBalanceSheet },
	{ id: 'entries', label: 'Entries', view: EntriesView, loader: loadEntries }
];
