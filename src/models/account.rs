use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize)]
pub struct Account {
    pub id: Uuid,
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub normal_balance: String,
    pub is_active: bool,
    pub parent_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct CreateAccountReq {
    pub company_id: Uuid,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub normal_balance: String,
    pub parent_id: Option<Uuid>,
}
