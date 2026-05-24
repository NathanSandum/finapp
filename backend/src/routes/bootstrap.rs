use axum::extract::Extension;
use serde_json::{json, Value};

use crate::auth::WorkspaceMember;
use crate::domain::entry::Entry;
use crate::domain::recurring::RecurringRule;
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
    let postings_by_entry = Entry::postings_for(&entries, &mut conn)?;
    let entry_items: Vec<Value> = entries
        .iter()
        .zip(postings_by_entry.iter())
        .map(|(entry, postings)| entry_to_json(entry, postings))
        .collect();

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
    let templates_by_rule = RecurringRule::templates_for(&rules, &mut conn)?;
    let rule_items = RecurringRuleResponse::build_batch(rules, templates_by_rule)?;

    Ok(JSend::success(json!({
        "entries": entry_items,
        "accounts": account_items,
        "anchors": anchors_json,
        "recurring_rules": rule_items,
    })))
}
