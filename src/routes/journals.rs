use axum::{Router, routing::{get, post}, extract::{State, Path}, Json};
use uuid::Uuid;

use crate::{app::AppState, error::AppError};
use crate::models::journal::{JournalEntry, CreateJournalReq};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_journal).get(list_journals))
        .route("/:id", get(get_journal))
        .route("/:id/post", post(post_journal))
}

async fn create_journal(
    State(st): State<AppState>,
    Json(req): Json<CreateJournalReq>,
) -> Result<Json<JournalEntry>, AppError> {
    // cek balance di sisi app (tetap ada cek DB saat posting)
    let debit: f64 = req.lines.iter().map(|l| l.debit).sum();
    let credit: f64 = req.lines.iter().map(|l| l.credit).sum();
    if (debit - credit).abs() > 0.0001 {
        return Err(AppError::BadRequest(format!(
            "Journal not balanced: debit={} credit={}",
            debit, credit
        )));
    }

    let mut tx = st.db.begin().await?;

    let entry = sqlx::query_as!(
        JournalEntry,
        r#"
        INSERT INTO journal_entries (company_id, entry_no, entry_date, memo, status)
        VALUES ($1,$2,$3,$4,'draft')
        RETURNING id, company_id, entry_no, entry_date, memo, status
        "#,
        req.company_id,
        req.entry_no,
        req.entry_date,
        req.memo
    )
    .fetch_one(&mut *tx)
    .await?;

    for l in req.lines {
        sqlx::query!(
            r#"
            INSERT INTO journal_lines (journal_entry_id, account_id, description, debit, credit)
            VALUES ($1,$2,$3,$4,$5)
            "#,
            entry.id,
            l.account_id,
            l.description,
            l.debit,
            l.credit
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Json(entry))
}

async fn list_journals(State(st): State<AppState>) -> Result<Json<Vec<JournalEntry>>, AppError> {
    let rows = sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT id, company_id, entry_no, entry_date, memo, status
        FROM journal_entries
        ORDER BY entry_date DESC, entry_no DESC
        "#
    )
    .fetch_all(&st.db)
    .await?;

    Ok(Json(rows))
}

async fn get_journal(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<JournalEntry>, AppError> {
    let row = sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT id, company_id, entry_no, entry_date, memo, status
        FROM journal_entries
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&st.db)
    .await?;

    row.map(Json).ok_or(AppError::NotFound)
}

async fn post_journal(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query!("SELECT post_journal_entry($1)", id)
        .execute(&st.db)
        .await?;

    Ok(Json(serde_json::json!({ "status": "posted", "id": id })))
}
