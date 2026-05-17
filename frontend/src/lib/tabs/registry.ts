import OverviewView from '$lib/views/overview/View.svelte';
import IncomeView from '$lib/views/income/View.svelte';
import CashFlowView from '$lib/views/cash-flow/View.svelte';
import BalanceSheetView from '$lib/views/balance-sheet/View.svelte';
import EntriesView from '$lib/views/entries/View.svelte';
import AccountsView from '$lib/views/accounts/ViewA.svelte';
import { load as loadEntries } from '$lib/views/entries/loader';
import { load as loadOverview } from '$lib/views/overview/loader';
import { load as loadIncome } from '$lib/views/income/loader';
import { load as loadCashFlow } from '$lib/views/cash-flow/loader';
import { load as loadBalanceSheet } from '$lib/views/balance-sheet/loader';
import { load as loadAccounts } from '$lib/views/accounts/loader';
import type { Tab } from './types';

export const tabs: Tab[] = [
	{
		id: 'overview',
		label: 'Overview',
		title: 'Overview',
		hasPeriodHeader: true,
		newEntryVerb: 'spent',
		view: OverviewView,
		loader: loadOverview
	},
	{
		id: 'income',
		label: 'Income',
		title: 'Income',
		hasPeriodHeader: true,
		newEntryVerb: 'spent',
		view: IncomeView,
		loader: loadIncome
	},
	{
		id: 'cash-flow',
		label: 'Cash Flow',
		title: 'Cash Flow',
		hasPeriodHeader: true,
		newEntryVerb: 'contributed',
		view: CashFlowView,
		loader: loadCashFlow
	},
	{
		id: 'balance-sheet',
		label: 'Balance Sheet',
		title: 'Balance Sheet',
		hasPeriodHeader: true,
		newEntryVerb: 'contributed',
		view: BalanceSheetView,
		loader: loadBalanceSheet
	},
	{
		id: 'accounts',
		label: 'Accounts',
		title: 'Accounts',
		hasPeriodHeader: true,
		newEntryVerb: 'spent',
		view: AccountsView,
		loader: loadAccounts
	},
	{
		id: 'entries',
		label: 'Entries',
		title: 'Entries',
		hasPeriodHeader: true,
		newEntryVerb: 'spent',
		view: EntriesView,
		loader: loadEntries
	}
];
