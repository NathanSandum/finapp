use diesel::SqliteConnection;

use crate::domain::account::{Account, AccountRef, AccountType};
use crate::domain::workspace::WorkspaceRef;
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
    pub as_of_date: String,
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
        workspace: &WorkspaceRef,
        date: &str,
        conn: &mut SqliteConnection,
    ) -> Result<BalanceSheet, AppError> {
        let roots = workspace.root_accounts(conn)?;

        let mut total_assets: i64 = 0;
        let mut total_liabilities: i64 = 0;
        let mut total_equity: i64 = 0;
        let mut revenue_total: i64 = 0;
        let mut expense_total: i64 = 0;

        let mut assets_detail = Vec::new();
        let mut liabilities_detail = Vec::new();
        let mut equity_detail = Vec::new();

        for root in &roots {
            let root_balance = root.balance_as_of_with_descendants(date, conn)?;

            match root.account_type {
                AccountType::Asset => {
                    total_assets = root_balance;
                    assets_detail = Self::children_balances(root, date, false, conn)?;
                }
                AccountType::Liability => {
                    total_liabilities = -root_balance;
                    liabilities_detail = Self::children_balances(root, date, true, conn)?;
                }
                AccountType::Equity => {
                    total_equity = -root_balance;
                    equity_detail = Self::children_balances(root, date, true, conn)?;
                }
                AccountType::Revenue => {
                    revenue_total = root_balance;
                }
                AccountType::Expense => {
                    expense_total = root_balance;
                }
            }
        }

        let net_income = -(revenue_total + expense_total);
        let total_liabilities_and_equity = total_liabilities + total_equity + net_income;

        Ok(BalanceSheet {
            as_of_date: date.to_string(),
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
        date: &str,
        negate: bool,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<AccountBalance>, AppError> {
        let children = root.children(conn)?;
        let mut result = Vec::with_capacity(children.len());
        for child in &children {
            let raw_balance = child.balance_as_of_with_descendants(date, conn)?;
            let balance = if negate { -raw_balance } else { raw_balance };
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
    pub start_date: String,
    pub end_date: String,
    pub revenue: Vec<AccountBalance>,
    pub expenses: Vec<AccountBalance>,
    pub total_revenue: i64,
    pub total_expenses: i64,
    pub net_income: i64,
}

impl IncomeStatement {
    pub fn for_period(
        workspace: &WorkspaceRef,
        start_date: &str,
        end_date: &str,
        conn: &mut SqliteConnection,
    ) -> Result<IncomeStatement, AppError> {
        let roots = workspace.root_accounts(conn)?;

        let mut revenue_balance: i64 = 0;
        let mut expense_balance: i64 = 0;
        let mut revenue_detail = Vec::new();
        let mut expense_detail = Vec::new();

        for root in &roots {
            match root.account_type {
                AccountType::Revenue => {
                    revenue_balance =
                        root.balance_between_with_descendants(start_date, end_date, conn)?;
                    revenue_detail =
                        Self::children_balances(root, start_date, end_date, true, conn)?;
                }
                AccountType::Expense => {
                    expense_balance =
                        root.balance_between_with_descendants(start_date, end_date, conn)?;
                    expense_detail =
                        Self::children_balances(root, start_date, end_date, false, conn)?;
                }
                _ => {}
            }
        }

        // Revenue is negative in ledger (credit-normal), negate for display
        let total_revenue = -revenue_balance;
        let total_expenses = expense_balance;
        let net_income = total_revenue - total_expenses;

        Ok(IncomeStatement {
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
            revenue: revenue_detail,
            expenses: expense_detail,
            total_revenue,
            total_expenses,
            net_income,
        })
    }

    fn children_balances(
        root: &Account,
        start_date: &str,
        end_date: &str,
        negate: bool,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<AccountBalance>, AppError> {
        let children = root.children(conn)?;
        let mut result = Vec::with_capacity(children.len());
        for child in &children {
            let raw_balance =
                child.balance_between_with_descendants(start_date, end_date, conn)?;
            let balance = if negate { -raw_balance } else { raw_balance };
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
            date,
            "test",
            vec![
                PostingInput::new(from.id.clone(), -amount),
                PostingInput::new(to.id.clone(), amount),
            ],
        )
        .cleared(true)
        .insert(conn)
        .unwrap();
    }

    // ── Balance Sheet tests ────────────────────────────────────────

    #[test]
    fn balance_sheet_all_zeros_when_no_entries() {
        let (mut conn, workspace) = setup();

        let sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-12-31", &mut conn).unwrap();

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

        let sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-01-31", &mut conn).unwrap();

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

        let jan_sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-01-31", &mut conn).unwrap();
        assert_eq!(jan_sheet.total_assets, 500000);

        let feb_sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-02-28", &mut conn).unwrap();
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

        let sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-01-31", &mut conn).unwrap();

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

        let sheet = BalanceSheet::as_of(workspace.as_ref(), "2026-01-31", &mut conn).unwrap();

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

        let statement = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();

        let employment_revenue = statement.revenue.iter().find(|a| a.name == "Employment").unwrap();
        assert_eq!(employment_revenue.balance, 500000, "Revenue detail should be display-positive");
    }

    // ── Income Statement tests ─────────────────────────────────────

    #[test]
    fn income_statement_all_zeros_when_no_entries() {
        let (mut conn, workspace) = setup();

        let statement = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();

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

        let statement = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();

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

        let jan = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();
        assert_eq!(jan.total_revenue, 500000);

        let both = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-02-28",
            &mut conn,
        )
        .unwrap();
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

        let statement = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();

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

        let statement = IncomeStatement::for_period(
            workspace.as_ref(),
            "2026-01-01",
            "2026-01-31",
            &mut conn,
        )
        .unwrap();

        // Food in detail includes Cafes (descendant rollup)
        let food_expense = statement.expenses.iter().find(|a| a.name == "Food").unwrap();
        assert_eq!(food_expense.balance, 1700);

        assert_eq!(statement.total_expenses, 1700);
    }
}
