use chrono::NaiveDate;
use diesel::SqliteConnection;

use crate::domain::account::{
    Account, AccountRef, BalanceSheetAnchors, IncomeStatementAnchors,
};
use crate::error::AppError;

// ── AccountBalance ──────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct AccountBalance {
    pub account: AccountRef,
    pub name: String,
    pub balance: i64,
}

// ── BalanceSheet ────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct BalanceSheet {
    pub as_of_date: NaiveDate,
    pub assets: Vec<AccountBalance>,
    pub liabilities: Vec<AccountBalance>,
    pub equity: Vec<AccountBalance>,
    pub total_assets: i64,
    pub total_liabilities: i64,
    pub total_equity: i64,
    pub net_income: i64,
    pub total_liabilities_and_equity: i64,
}

impl BalanceSheet {
    pub fn as_of(
        anchors: BalanceSheetAnchors<'_>,
        date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<BalanceSheet, AppError> {
        let total_assets = anchors.assets.balance_as_of_with_descendants(date, conn)?;
        let total_liabilities = -anchors.liabilities.balance_as_of_with_descendants(date, conn)?;
        let total_equity = -anchors.equity.balance_as_of_with_descendants(date, conn)?;
        let revenue_total = anchors.revenue.balance_as_of_with_descendants(date, conn)?;
        let expense_total = anchors.expenses.balance_as_of_with_descendants(date, conn)?;

        let assets_detail = Self::children_balances(anchors.assets, date, false, conn)?;
        let liabilities_detail = Self::children_balances(anchors.liabilities, date, true, conn)?;
        let equity_detail = Self::children_balances(anchors.equity, date, true, conn)?;

        let net_income = -(revenue_total + expense_total);
        let total_liabilities_and_equity = total_liabilities + total_equity + net_income;

        Ok(BalanceSheet {
            as_of_date: date,
            assets: assets_detail,
            liabilities: liabilities_detail,
            equity: equity_detail,
            total_assets,
            total_liabilities,
            total_equity,
            net_income,
            total_liabilities_and_equity,
        })
    }

    fn children_balances(
        root: &Account,
        date: NaiveDate,
        negate: bool,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<AccountBalance>, AppError> {
        let children = root.children(conn)?;
        let mut result = Vec::with_capacity(children.len());
        for child in &children {
            let raw_balance = child.balance_as_of_with_descendants(date, conn)?;
            let balance = if negate { -raw_balance } else { raw_balance };
            if balance == 0 {
                continue;
            }
            result.push(AccountBalance {
                account: child.id.clone(),
                name: child.name.clone(),
                balance,
            });
        }
        Ok(result)
    }
}

// ── IncomeStatement ────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct IncomeStatement {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub revenue: Vec<AccountBalance>,
    pub expenses: Vec<AccountBalance>,
    pub total_revenue: i64,
    pub total_expenses: i64,
    pub net_income: i64,
}

impl IncomeStatement {
    pub fn for_period(
        anchors: IncomeStatementAnchors<'_>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        conn: &mut SqliteConnection,
    ) -> Result<IncomeStatement, AppError> {
        let revenue_balance =
            anchors.revenue.balance_between_with_descendants(start_date, end_date, conn)?;
        let expense_balance =
            anchors.expenses.balance_between_with_descendants(start_date, end_date, conn)?;
        let revenue_detail =
            Self::children_balances(anchors.revenue, start_date, end_date, true, conn)?;
        let expense_detail =
            Self::children_balances(anchors.expenses, start_date, end_date, false, conn)?;

        // Revenue is negative in ledger (credit-normal), negate for display
        let total_revenue = -revenue_balance;
        let total_expenses = expense_balance;
        let net_income = total_revenue - total_expenses;

        Ok(IncomeStatement {
            start_date,
            end_date,
            revenue: revenue_detail,
            expenses: expense_detail,
            total_revenue,
            total_expenses,
            net_income,
        })
    }

    fn children_balances(
        root: &Account,
        start_date: NaiveDate,
        end_date: NaiveDate,
        negate: bool,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<AccountBalance>, AppError> {
        let children = root.children(conn)?;
        let mut result = Vec::with_capacity(children.len());
        for child in &children {
            let raw_balance =
                child.balance_between_with_descendants(start_date, end_date, conn)?;
            let balance = if negate { -raw_balance } else { raw_balance };
            if balance == 0 {
                continue;
            }
            result.push(AccountBalance {
                account: child.id.clone(),
                name: child.name.clone(),
                balance,
            });
        }
        Ok(result)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::account::{Account, NewAccount};
    use crate::domain::entry::{NewEntry, PostingInput};
    use crate::domain::user::NewUser;
    use crate::domain::workspace::{NewWorkspace, Workspace};
    use crate::test_helpers::setup_test_db;

    fn setup() -> (SqliteConnection, Workspace) {
        let mut conn = setup_test_db();
        let user = NewUser::new("alice@example.com", "Alice", "password")
            .unwrap()
            .insert(&mut conn)
            .unwrap();
        let workspace = NewWorkspace::new("My Finances", user.as_ref().clone())
            .insert(&mut conn)
            .unwrap();
        (conn, workspace)
    }

    fn find_account(name: &str, workspace: &Workspace, conn: &mut SqliteConnection) -> Account {
        let all = workspace.as_ref().accounts(conn).unwrap();
        all.into_iter().find(|a| a.name == name).unwrap()
    }

    fn nd(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    fn balance_sheet_for(
        workspace: &Workspace,
        date: &str,
        conn: &mut SqliteConnection,
    ) -> BalanceSheet {
        let anchors = workspace.as_ref().anchors(conn).unwrap();
        BalanceSheet::as_of(anchors.balance_sheet(), nd(date), conn).unwrap()
    }

    fn income_statement_for(
        workspace: &Workspace,
        start_date: &str,
        end_date: &str,
        conn: &mut SqliteConnection,
    ) -> IncomeStatement {
        let anchors = workspace.as_ref().anchors(conn).unwrap();
        IncomeStatement::for_period(anchors.income_statement(), nd(start_date), nd(end_date), conn)
            .unwrap()
    }

    fn make_entry(
        workspace: &Workspace,
        from: &Account,
        to: &Account,
        amount: i32,
        date: &str,
        conn: &mut SqliteConnection,
    ) {
        NewEntry::new(
            workspace.as_ref().clone(),
            nd(date),
            "test",
            vec![
                PostingInput::new(from.id.clone(), -amount),
                PostingInput::new(to.id.clone(), amount),
            ],
        )
        .insert(conn)
        .unwrap();
    }

    // ── Balance Sheet tests ────────────────────────────────────────

    #[test]
    fn balance_sheet_all_zeros_when_no_entries() {
        let (mut conn, workspace) = setup();

        let sheet = balance_sheet_for(&workspace,"2026-12-31", &mut conn);

        assert_eq!(sheet.total_assets, 0);
        assert_eq!(sheet.total_liabilities, 0);
        assert_eq!(sheet.total_equity, 0);
        assert_eq!(sheet.net_income, 0);
        assert_eq!(sheet.total_liabilities_and_equity, 0);
    }

    #[test]
    fn balance_sheet_salary_and_expense() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        // Earn 5000.00
        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);
        // Spend 12.00 on food
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-05", &mut conn);

        let sheet = balance_sheet_for(&workspace,"2026-01-31", &mut conn);

        // Assets = 5000.00 - 12.00 = 4988.00
        assert_eq!(sheet.total_assets, 498800);
        assert_eq!(sheet.total_liabilities, 0);
        assert_eq!(sheet.total_equity, 0);
        // Net income = -((-500000) + 1200) = 498800
        assert_eq!(sheet.net_income, 498800);
        // Accounting equation: A = L + E + NI
        assert_eq!(sheet.total_assets, sheet.total_liabilities_and_equity);
    }

    #[test]
    fn balance_sheet_respects_as_of_date() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-15", &mut conn);
        make_entry(&workspace, &employment, &chequing, 300000, "2026-02-15", &mut conn);

        let jan_sheet = balance_sheet_for(&workspace,"2026-01-31", &mut conn);
        assert_eq!(jan_sheet.total_assets, 500000);

        let feb_sheet = balance_sheet_for(&workspace,"2026-02-28", &mut conn);
        assert_eq!(feb_sheet.total_assets, 800000);
    }

    #[test]
    fn balance_sheet_per_account_detail() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let savings = find_account("Savings", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);
        make_entry(&workspace, &employment, &savings, 200000, "2026-01-01", &mut conn);

        let sheet = balance_sheet_for(&workspace,"2026-01-31", &mut conn);

        let chequing_balance = sheet.assets.iter().find(|a| a.name == "Chequing").unwrap();
        assert_eq!(chequing_balance.balance, 500000);

        let savings_balance = sheet.assets.iter().find(|a| a.name == "Savings").unwrap();
        assert_eq!(savings_balance.balance, 200000);
    }

    #[test]
    fn balance_sheet_accounting_equation_holds() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let credit_card = find_account("Credit Card", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        // Earn salary
        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);
        // Pay with credit card
        make_entry(&workspace, &credit_card, &food, 5000, "2026-01-05", &mut conn);
        // Pay rent from chequing
        make_entry(&workspace, &chequing, &rent, 150000, "2026-01-05", &mut conn);

        let sheet = balance_sheet_for(&workspace,"2026-01-31", &mut conn);

        // Liabilities display-positive (credit card owes 50.00)
        assert_eq!(sheet.total_liabilities, 5000);
        let cc_balance = sheet.liabilities.iter().find(|a| a.name == "Credit Card").unwrap();
        assert_eq!(cc_balance.balance, 5000);

        assert_eq!(
            sheet.total_assets, sheet.total_liabilities_and_equity,
            "A={} != L+E+NI={}",
            sheet.total_assets, sheet.total_liabilities_and_equity
        );
    }

    #[test]
    fn income_statement_revenue_detail_display_positive() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        let employment_revenue = statement.revenue.iter().find(|a| a.name == "Employment").unwrap();
        assert_eq!(employment_revenue.balance, 500000, "Revenue detail should be display-positive");
    }

    // ── Income Statement tests ─────────────────────────────────────

    #[test]
    fn income_statement_all_zeros_when_no_entries() {
        let (mut conn, workspace) = setup();

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        assert_eq!(statement.total_revenue, 0);
        assert_eq!(statement.total_expenses, 0);
        assert_eq!(statement.net_income, 0);
    }

    #[test]
    fn income_statement_salary_and_expense() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-05", &mut conn);

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        assert_eq!(statement.total_revenue, 500000);
        assert_eq!(statement.total_expenses, 1200);
        assert_eq!(statement.net_income, 498800);
    }

    #[test]
    fn income_statement_respects_date_range() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-15", &mut conn);
        make_entry(&workspace, &employment, &chequing, 300000, "2026-02-15", &mut conn);

        let jan = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);
        assert_eq!(jan.total_revenue, 500000);

        let both = income_statement_for(&workspace, "2026-01-01", "2026-02-28", &mut conn);
        assert_eq!(both.total_revenue, 800000);
    }

    #[test]
    fn income_statement_per_account_detail() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);
        let rent = find_account("Rent", &workspace, &mut conn);

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-05", &mut conn);
        make_entry(&workspace, &chequing, &rent, 150000, "2026-01-05", &mut conn);

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        let food_expense = statement.expenses.iter().find(|a| a.name == "Food").unwrap();
        assert_eq!(food_expense.balance, 1200);

        let rent_expense = statement.expenses.iter().find(|a| a.name == "Rent").unwrap();
        assert_eq!(rent_expense.balance, 150000);
    }

    #[test]
    fn income_statement_sub_accounts_aggregate() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        let cafes = NewAccount::child(&food, "Cafes").insert(&mut conn).unwrap();

        make_entry(&workspace, &chequing, &food, 1200, "2026-01-05", &mut conn);
        make_entry(&workspace, &chequing, &cafes, 500, "2026-01-06", &mut conn);

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        // Food in detail includes Cafes (descendant rollup)
        let food_expense = statement.expenses.iter().find(|a| a.name == "Food").unwrap();
        assert_eq!(food_expense.balance, 1700);

        assert_eq!(statement.total_expenses, 1700);
    }

    #[test]
    fn balance_sheet_excludes_zero_balance_accounts() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);

        // Only Chequing gets a balance; Savings, TFSA, RRSP stay at zero
        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);

        let sheet = balance_sheet_for(&workspace,"2026-01-31", &mut conn);

        // Should contain Chequing
        assert!(sheet.assets.iter().any(|a| a.name == "Chequing"));
        // Should NOT contain zero-balance accounts
        assert!(!sheet.assets.iter().any(|a| a.name == "Savings"));
        assert!(!sheet.assets.iter().any(|a| a.name == "TFSA"));
        assert!(!sheet.assets.iter().any(|a| a.name == "RRSP"));
        // Totals still correct
        assert_eq!(sheet.total_assets, 500000);
    }

    #[test]
    fn income_statement_excludes_zero_balance_accounts() {
        let (mut conn, workspace) = setup();
        let chequing = find_account("Chequing", &workspace, &mut conn);
        let employment = find_account("Employment", &workspace, &mut conn);
        let food = find_account("Food", &workspace, &mut conn);

        make_entry(&workspace, &employment, &chequing, 500000, "2026-01-01", &mut conn);
        make_entry(&workspace, &chequing, &food, 1200, "2026-01-05", &mut conn);

        let statement = income_statement_for(&workspace, "2026-01-01", "2026-01-31", &mut conn);

        // Revenue should only contain Employment
        assert_eq!(statement.revenue.len(), 1);
        assert_eq!(statement.revenue[0].name, "Employment");
        // Expenses should only contain Food (not Rent, Groceries, etc.)
        assert_eq!(statement.expenses.len(), 1);
        assert_eq!(statement.expenses[0].name, "Food");
        // Totals still correct
        assert_eq!(statement.total_revenue, 500000);
        assert_eq!(statement.total_expenses, 1200);
    }
}
