use std::collections::HashMap;

use axum::{extract::Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::account::{Account, AccountType, NewAccount};
use crate::domain::category::{Category, NewCategory};
use crate::domain::transaction::{LineItemInput, NewTransaction};
use crate::domain::workspace::Role;
use crate::error::AppError;
use crate::DbPool;

#[derive(Deserialize)]
pub struct ImportRequest {
    transactions: Vec<ImportTransaction>,
}

#[derive(Deserialize)]
pub struct ImportTransaction {
    account: String,
    account_type: Option<String>,
    date: String,
    description: Option<String>,
    amount: i32,
    category: String,
    cleared: Option<bool>,
}

pub async fn import(
    Extension(pool): Extension<DbPool>,
    member: WorkspaceMember,
    Json(body): Json<ImportRequest>,
) -> Result<Json<Value>, AppError> {
    member.require_role(&Role::Editor)?;
    let mut conn = pool.get().map_err(|err| AppError::Internal(format!("db pool: {err}")))?;

    // Load existing accounts and categories for this workspace
    let existing_accounts = Account::for_workspace(&member.workspace, &mut conn)?;
    let existing_categories = Category::for_workspace(&member.workspace, &mut conn)?;

    let mut account_map: HashMap<String, Account> = existing_accounts
        .into_iter()
        .map(|a| (a.name.clone(), a))
        .collect();

    let mut category_map: HashMap<String, Category> = existing_categories
        .into_iter()
        .map(|c| (c.name.clone(), c))
        .collect();

    let mut accounts_created: u32 = 0;
    let mut categories_created: u32 = 0;
    let mut transactions_created: u32 = 0;

    // Collect unique accounts and categories to create
    for txn in &body.transactions {
        if !account_map.contains_key(&txn.account) {
            let account_type = txn
                .account_type
                .as_deref()
                .and_then(AccountType::from_str)
                .unwrap_or(AccountType::Other);

            let account = NewAccount::new(&member.workspace, &txn.account, account_type)
                .insert(&mut conn)?;
            account_map.insert(txn.account.clone(), account);
            accounts_created += 1;
        }

        if !category_map.contains_key(&txn.category) {
            let category = NewCategory::new(&member.workspace, &txn.category)
                .insert(&mut conn)?;
            category_map.insert(txn.category.clone(), category);
            categories_created += 1;
        }
    }

    // Create transactions
    for txn in &body.transactions {
        let account = &account_map[&txn.account];
        let category = &category_map[&txn.category];

        let line_item = LineItemInput::new(category, txn.amount);

        let mut builder = NewTransaction::new(
            &member.workspace,
            account,
            &txn.date,
            txn.amount,
            vec![line_item],
        );

        if let Some(ref desc) = txn.description {
            builder = builder.description(desc);
        }

        if let Some(cleared) = txn.cleared {
            builder = builder.cleared(cleared);
        }

        builder.insert(&mut conn)?;
        transactions_created += 1;
    }

    Ok(Json(json!({
        "accounts_created": accounts_created,
        "categories_created": categories_created,
        "transactions_created": transactions_created,
    })))
}
