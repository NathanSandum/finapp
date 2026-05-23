use axum::extract::Extension;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::error::AppError;
use crate::jsend::JSend;
use crate::routes::accounts::account_to_json;
use crate::routes::entries::entry_to_json;
use crate::routes::recurring::RecurringRuleResponse;
use crate::DbPool;

/// Returns every resource a freshly-booted client needs in a single response.
/// Saves four round-trips (entries, accounts, anchors, recurring) on dashboard
/// mount; individual endpoints stay live for post-mutation refetches.
pub async fn bootstrap(
    Extension(db): Extension<DbPool>,
    member: WorkspaceMember,
) -> Result<JSend<Value>, AppError> {
    let mut conn = db
        .get()
        .map_err(|err| AppError::Internal(format!("db: {err}")))?;

    let entries = member.workspace.entries(&mut conn)?;
    let mut entry_items = Vec::with_capacity(entries.len());
    for entry in &entries {
        let postings = entry.postings(&mut conn)?;
        entry_items.push(entry_to_json(entry, &postings));
    }

    let accounts = member.workspace.accounts(&mut conn)?;
    let account_items: Vec<Value> = accounts.iter().map(account_to_json).collect();

    let anchors = member.workspace.anchors(&mut conn)?;
    let anchors_json = json!({
        "assets": account_to_json(&anchors.assets),
        "liabilities": account_to_json(&anchors.liabilities),
        "equity": account_to_json(&anchors.equity),
        "revenue": account_to_json(&anchors.revenue),
        "expenses": account_to_json(&anchors.expenses),
        "cash": account_to_json(&anchors.cash),
        "opening_equity": account_to_json(&anchors.opening_equity),
    });

    let rules = member.workspace.recurring_rules(&mut conn)?;
    let mut rule_items = Vec::with_capacity(rules.len());
    for rule in rules {
        rule_items.push(RecurringRuleResponse::build(rule, &mut conn)?);
    }

    Ok(JSend::success(json!({
        "entries": entry_items,
        "accounts": account_items,
        "anchors": anchors_json,
        "recurring_rules": rule_items,
    })))
}
