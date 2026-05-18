use axum::{Router, routing::{get, post}, extract::{State, Path}, Json};
use uuid::Uuid;

use crate::{app::AppState, error::AppError};
use crate::models::account::{Account, CreateAccountReq};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_account).get(list_accounts))
        .route("/:id", get(get_account))
}

async fn create_account(
    State(st): State<AppState>,
    Json(req): Json<CreateAccountReq>,
) -> Result<Json<Account>, AppError> {
    let rec = sqlx::query_as!(
        Account,
        r#"
        INSERT INTO accounts (company_id, code, name, account_type, normal_balance, parent_id)
        VALUES ($1,$2,$3,$4,$5,$6)
        RETURNING id, company_id, code, name, account_type, normal_balance, is_active, parent_id
        "#,
        req.company_id,
        req.code,
        req.name,
        req.account_type,
        req.normal_balance,
        req.parent_id
    )
    .fetch_one(&st.db)
    .await?;

    Ok(Json(rec))
}

async fn list_accounts(
    State(st): State<AppState>,
) -> Result<Json<Vec<Account>>, AppError> {
    let rows = sqlx::query_as!(
        Account,
        r#"
        SELECT id, company_id, code, name, account_type, normal_balance, is_active, parent_id
        FROM accounts
        ORDER BY code
        "#
    )
    .fetch_all(&st.db)
    .await?;

    Ok(Json(rows))
}

async fn get_account(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Account>, AppError> {
    let row = sqlx::query_as!(
        Account,
        r#"
        SELECT id, company_id, code, name, account_type, normal_balance, is_active, parent_id
        FROM accounts
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(&st.db)
    .await?;

    match row {
        Some(a) => Ok(Json(a)),
        None => Err(AppError::NotFound),
    }
}
