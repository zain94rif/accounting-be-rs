use axum::{Router, routing::{get, post}, extract::{State, Path}, Json};
use uuid::Uuid;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{app::AppState, error::AppError};
use crate::models::journal::{JournalEntry, CreateJournalReq};

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

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
    // Cek balance di sisi app (tetap ada cek DB saat posting)
    let debit: Decimal = req.lines.iter().map(|l| l.debit).sum();
    let credit: Decimal = req.lines.iter().map(|l| l.credit).sum();
    if debit != credit {
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

    // Bulk insert menggunakan QueryBuilder
    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO journal_lines (journal_entry_id, account_id, description, debit, credit) "
    );

    query_builder.push_values(req.lines, |mut b, line| {
        b.push_bind(entry.id)
         .push_bind(line.account_id)
         .push_bind(line.description)
         .push_bind(line.debit)
         .push_bind(line.credit);
    });

    let query = query_builder.build();
    query.execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(Json(entry))
}

async fn list_journals(
    State(st): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> Result<Json<Vec<JournalEntry>>, AppError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let rows = sqlx::query_as!(
        JournalEntry,
        r#"
        SELECT id, company_id, entry_no, entry_date, memo, status
        FROM journal_entries
        ORDER BY entry_date DESC, entry_no DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
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
