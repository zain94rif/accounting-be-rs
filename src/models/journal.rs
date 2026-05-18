use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Serialize)]
pub struct JournalEntry {
    pub id: Uuid,
    pub company_id: Uuid,
    pub entry_no: String,
    pub entry_date: NaiveDate,
    pub memo: Option<String>,
    pub status: String,
}

#[derive(Deserialize)]
pub struct CreateJournalReq {
    pub company_id: Uuid,
    pub entry_no: String,
    pub entry_date: NaiveDate,
    pub memo: Option<String>,
    pub lines: Vec<CreateJournalLineReq>,
}

#[derive(Deserialize)]
pub struct CreateJournalLineReq {
    pub account_id: Uuid,
    pub description: Option<String>,
    pub debit: f64,
    pub credit: f64,
}
